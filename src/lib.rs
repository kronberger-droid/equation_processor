//! Core library for the Equation Processor.
//!
//! Provides parsing of equation files (CSV & Markdown), representation of equations,
//! and rendering to PDF/SVG via external tools (tectonic & pdftocairo),
//! with optional CLI progress indication.

pub use self::core::*;

mod core {
    use indicatif::{ProgressBar, ProgressStyle};
    use prettytable::{row, Table};
    use regex::Regex;
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::{self, BufRead, BufReader, Read, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    /// Supported input file types.
    #[derive(Debug)]
    pub enum Filetype {
        /// CSV with header [active,equation,name]
        Csv,
        /// Markdown: $$...$$ blocks, optional %%yes%%/%%no%% and %%name%% tags
        Markdown,
        /// Unknown or unsupported extension
        Unknown,
    }

    /// A mathematical equation entry.
    #[derive(Debug, Clone)]
    pub struct Equation {
        /// Whether to render this equation
        pub active: bool,
        /// Filename-safe name for output files
        pub name: String,
        /// LaTeX body of the equation
        pub body: String,
    }

    impl Equation {
        /// Construct new equation, sanitizing name
        pub fn new(active: bool, name: &str, body: &str) -> Self {
            let sanitized = Equation::sanitize_filename(name);
            Equation {
                active,
                name: sanitized,
                body: body.to_string(),
            }
        }

        /// Replace invalid characters with underscores
        fn sanitize_filename(name: &str) -> String {
            let re = Regex::new(r"[^A-Za-z0-9_.]").unwrap();
            let mut s = re.replace_all(name, "_").into_owned();
            if s.is_empty() {
                s = "default_equation".into();
            }
            s
        }

        /// Render to PDF and SVG, optionally cleaning up _aux files
        pub fn render(
            &self,
            output_dir: &PathBuf,
            color: &str,
            delete_intermediates: bool,
        ) -> io::Result<()> {
            if !self.active {
                return Ok(());
            }
            fs::create_dir_all(output_dir)?;
            let tex = self.generate_latex(color);
            let tex_path = output_dir.join(format!("{}.tex", self.name));
            fs::write(&tex_path, tex)?;

            let status = Command::new("tectonic")
                .arg(&tex_path)
                .arg("--outdir")
                .arg(output_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()?;

            if status.success() {
                self.convert_pdf_to_svg(output_dir)?;
                if delete_intermediates {
                    self.cleanup_intermediate_files(output_dir)?;
                }
            }
            Ok(())
        }

        /// Convert the .pdf to .svg
        fn convert_pdf_to_svg(&self, output_dir: &Path) -> io::Result<()> {
            let pdf = output_dir.join(format!("{}.pdf", self.name));
            let svg = output_dir.join(format!("{}.svg", self.name));
            let status = Command::new("pdftocairo")
                .arg("-svg")
                .arg(&pdf)
                .arg(&svg)
                .status()?;
            if status.success() {
                Ok(())
            } else {
                Err(io::Error::other("SVG conversion failed"))
            }
        }

        /// Remove .tex and .pdf intermediates
        fn cleanup_intermediate_files(&self, output_dir: &Path) -> io::Result<()> {
            let _ = fs::remove_file(output_dir.join(format!("{}.tex", self.name)));
            let _ = fs::remove_file(output_dir.join(format!("{}.pdf", self.name)));
            Ok(())
        }

        /// Generate LaTeX source including custom font and color
        fn generate_latex(&self, color: &str) -> String {
            let code = color.trim_start_matches('#');
            format!(
                r#"\documentclass[border=1pt]{{standalone}}
                \usepackage{{amsmath}}
                \usepackage{{xfrac}}
                \usepackage{{gfsneohellenicot}}
                \usepackage{{xcolor}}
                \definecolor{{equationcolor}}{{HTML}}{{{}}}
                \begin{{document}}
                \setbox0\hbox{{\Large \textcolor{{equationcolor}}{{${}$}}}}
                \dimen0=12mm
                \ifdim\ht0<\dimen0 \ht0=\dimen0 \fi
                \ifdim\dp0<5mm \dp0=5mm \fi
                \box0
                \end{{document}}"#,
                code, self.body
            )
        }
    }

    /// Prompt user for yes/no on CLI
    pub fn ask_confirmation(prompt: &str) -> bool {
        loop {
            print!("{prompt} (y/n): ");
            io::stdout().flush().unwrap();
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            match buf.trim().to_lowercase().as_str() {
                "y" | "yes" => return true,
                "n" | "no" => return false,
                _ => continue,
            }
        }
    }

    /// Render all active equations with a CLI progress bar
    pub fn render_equations(
        equations: &[Equation],
        output_dir: &PathBuf,
        color: &str,
        delete_intermediates: bool,
    ) -> io::Result<()> {
        let active: Vec<&Equation> = equations.iter().filter(|e| e.active).collect();
        let bar = ProgressBar::new(active.len() as u64).with_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        for eq in active {
            bar.set_message(eq.name.clone());
            eq.render(output_dir, color, delete_intermediates)?;
            bar.inc(1);
        }
        bar.finish();
        Ok(())
    }

    /// Read file to string
    pub fn read_file(path: &PathBuf) -> io::Result<String> {
        let mut f = File::open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        Ok(s)
    }

    /// Parse CSV into equations
    pub fn read_csv_file(path: &PathBuf) -> io::Result<Vec<Equation>> {
        let f = File::open(path)?;
        let rdr = BufReader::new(f);
        let mut eqs = Vec::new();
        let mut counts = HashMap::new();
        for line in rdr.lines().skip(1).flatten() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let active = parts[0].trim().eq_ignore_ascii_case("yes");
                let body = parts[1].trim();
                let mut name = parts[2].trim().to_string();
                let c = counts.entry(name.clone()).or_insert(0);
                if *c > 0 {
                    name = format!("{name}_{c}");
                }
                *c += 1;
                eqs.push(Equation::new(active, &name, body));
            }
        }
        Ok(eqs)
    }

    /// Determine file type by extension
    pub fn detect_file_type(path: &Path) -> Filetype {
        match path.extension().and_then(|e| e.to_str()) {
            Some("csv") => Filetype::Csv,
            Some("md") | Some("markdown") => Filetype::Markdown,
            _ => Filetype::Unknown,
        }
    }

    /// Parse Markdown into equations
    pub fn parse_markdown(content: &str) -> Vec<Equation> {
        let re = Regex::new(r"(?s)(%%(yes|no)?%%)?[\n\r]*\$\$[\n\r]*(.*?)\$\$[\n\r]*(%%(.*?)%%)?")
            .unwrap();
        let mut eqs = Vec::new();
        let mut counts = HashMap::new();
        for cap in re.captures_iter(content) {
            let active = cap.get(2).is_none_or(|m| m.as_str() == "yes");
            let body = cap.get(3).unwrap().as_str().trim();
            let raw = cap.get(5).map_or("default_equation", |m| m.as_str());
            let c = counts.entry(raw.to_string()).or_insert(0);
            let name = if *c > 0 {
                format!("{raw}_{c}")
            } else {
                raw.to_string()
            };
            *c += 1;
            eqs.push(Equation::new(active, &name, body));
        }
        eqs
    }

    /// CLI entry: display table, confirm, then render.
    pub fn run_cli(
        input_file: PathBuf,
        color: &str,
        output_dir: &PathBuf,
        delete_intermediates: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(output_dir)?;
        let ft = detect_file_type(&input_file);
        let content = read_file(&input_file)?;
        let equations = match ft {
            Filetype::Csv => read_csv_file(&input_file)?,
            Filetype::Markdown => parse_markdown(&content),
            _ => return Err("Unsupported file type".into()),
        };
        if equations.is_empty() {
            println!("No equations found.");
            return Ok(());
        }
        // Show table
        let mut table = Table::new();
        table.add_row(row!["Active", "Name", "Equation"]);
        for eq in &equations {
            table.add_row(row![if eq.active { "Yes" } else { "No" }, eq.name, eq.body]);
        }
        table.printstd();

        if !ask_confirmation("Render active equations?") {
            return Ok(());
        }
        render_equations(&equations, output_dir, color, delete_intermediates)?;
        println!("Rendered to {output_dir:?}");
        Ok(())
    }

    /// Print a short table summary.
    pub fn display_table(equations: &[Equation]) {
        let mut table = Table::new();
        table.add_row(row!["Active", "Name", "Equation"]);
        for eq in equations {
            table.add_row(row![if eq.active { "Yes" } else { "No" }, eq.name, eq.body]);
        }
        table.printstd();
    }
}
