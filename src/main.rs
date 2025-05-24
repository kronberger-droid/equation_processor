//! ```no_run
//! # Equation Processor
//!
//! This application processes mathematical equations either via a command-line interface (CLI)
//! or a graphical user interface (GUI). By default, if no input file is provided, the GUI will launch.
//! Passing an input file path enables CLI mode for unattended batch processing.
//! ```

use clap::Parser;
use equation_processor::run_cli;
use std::process;
mod gui;

/// Command-line arguments for the Equation Processor.
///
/// - If `input_file` is provided, runs in CLI mode:
///   - Reads and parses equations from the specified file.
///   - Renders active equations to the output directory with the chosen color.
///   - Optionally deletes intermediate files.
/// - If no `input_file` is provided, launches the GUI application.
#[derive(Parser)]
#[command(
    name = "Equation Processor",
    about = "Run in CLI mode if an input file is given; otherwise launch GUI",
    version = "1.0"
)]
struct Args {
    /// Optional path to the input file containing equations.
    ///
    /// Supported formats:
    /// - CSV: Expect columns [active, equation, name]
    /// - Markdown: Delimited by `$$...$$` blocks, optional `%%yes%%`/`%%no%%` for active.
    #[arg(short, long, value_name = "INPUT_FILE")]
    input_file: Option<std::path::PathBuf>,

    /// Hex color code for rendered output (e.g., `#000000` for black).
    #[arg(short, long, default_value = "#000000")]
    color: String,

    /// Output directory for rendered files.
    #[arg(short, long, default_value = "./output")]
    output_dir: std::path::PathBuf,

    /// Delete intermediate LaTeX/PDF files after rendering.
    #[arg(short, long)]
    delete_intermediates: bool,
}

/// Entry point.
///
/// Parses arguments and either:
/// - Calls `run_cli(...)` to process equations in batch (CLI mode), or
/// - Launches the eframe GUI (`gui::launch_gui()`) if no input file was specified.
fn main() {
    // Parse and validate arguments
    let args = Args::parse();

    match args.input_file {
        Some(path) => {
            // CLI mode: delegate to library and exit on error
            if let Err(e) = run_cli(
                path,
                &args.color,
                &args.output_dir,
                args.delete_intermediates,
            ) {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
        None => {
            // GUI mode: start the interactive window
            gui::launch_gui();
        }
    }
}
