use clap::Parser;
use std::path::PathBuf;
use equation_processor::{read_file, read_csv_file, detect_file_type, parse_markdown, render_equations, ask_confirmation};
use prettytable::{Table, row};

#[derive(Parser)]
#[command(name = "Equation Processor")]
#[command(about = "Processes equations from input files", version = "1.0")]
struct Args {
    #[arg(short, long)]
    input_file: PathBuf,

    #[arg(short, long, default_value = "#000000")]
    color: String,

    #[arg(short, long, default_value = "./output")]
    output_dir: PathBuf,

    #[arg(short, long)]
    delete_intermediates: bool,
}

fn main() {
    let args = Args::parse();
    let file_type = detect_file_type(&args.input_file);

    std::fs::create_dir_all(&args.output_dir).unwrap();

    let equations = match file_type {
        "csv" => read_csv_file(&args.input_file).unwrap_or_else(|_| Vec::new()),
        "markdown" => {
            let content = read_file(&args.input_file).unwrap();
            parse_markdown(&content)
        }
        _ => {
            eprintln!("Unsupported file type.");
            return;
        }
    };

    if equations.is_empty() {
        eprintln!("No equations found to process.");
    } else {
        let mut table = Table::new();
        
        table.add_row(row!["Active", "Name", "Equation"]);

        for eq in &equations {
            table.add_row(row![
                if eq.active { "Yes" } else { "No" },
                eq.name,
                eq.body
            ]);
        }

        table.printstd();

        if !ask_confirmation("Are you sure you want to render the active equations?") {
            return
        }
        
        render_equations(&equations, &args.output_dir, &args.color, args.delete_intermediates).unwrap();
        
        println!(
            "  Equations rendered successfully to {:?}",
            args.output_dir
        );
    }
}
