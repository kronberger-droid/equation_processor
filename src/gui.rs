//! GUI module for the Equation Processor.
//!
//! This module provides an interactive graphical interface built with
//! [`eframe`](https://docs.rs/eframe) and [`egui`](https://docs.rs/egui) that allows
//! users to load equation files, configure rendering options, and execute
//! batch rendering with visual feedback.

use eframe::egui;
use eframe::egui::widgets::Spinner;
use eframe::egui::Color32;
use eframe::egui::{ScrollArea, ViewportBuilder};
use egui_extras::{Column, TableBuilder};
use egui_file_dialog::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use equation_processor::{detect_file_type, parse_markdown, read_csv_file, Equation, Filetype};

/// Holds the entire state for the GUI application.
///
/// Fields include configuration (input/output paths, colors, flags),
/// the loaded equations, rendering state (in progress or not),
/// file dialogs, and messages for errors or success.
#[derive(Default)]
pub struct EquationProcessorApp {
    /// Path to the currently selected input file, if any.
    input_file: Option<PathBuf>,
    /// Path to the selected output directory, if any.
    output_dir: Option<PathBuf>,
    /// RGB color for equation rendering, normalized to [0.0,1.0].
    font_color: [f32; 3],
    /// Hex color string for text input.
    color_hex_input: String,
    /// Flag to delete intermediate LaTeX/PDF files.
    delete_intermediates: bool,
    /// Vector of equations parsed from the input file.
    equations: Vec<Equation>,
    /// Whether a rendering operation is currently in progress.
    processing: bool,
    /// Receiver channel used to signal completion of the background render.
    progress_rx: Option<mpsc::Receiver<()>>,
    /// File dialog for selecting the input file.
    open_file_dialog: FileDialog,
    /// Directory dialog for selecting the output directory.
    select_dir_dialog: FileDialog,
    /// Optional error message to display in red.
    error_message: Option<String>,
    /// Optional success message to display in green.
    success_message: Option<String>,
}

impl EquationProcessorApp {
    /// Constructs the `EquationProcessorApp` and initializes dialogs and defaults.
    ///
    /// This sets up the file and directory dialogs and default values for
    /// color and flags. Other fields use their `Default` values.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            open_file_dialog: FileDialog::new(),
            select_dir_dialog: FileDialog::new(),
            font_color: [0.0, 0.0, 0.0],
            color_hex_input: "#000000".to_string(),
            ..Default::default()
        }
    }

    /// Convert RGB float array to hex string using egui's Color32
    fn rgb_to_hex(rgb: [f32; 3]) -> String {
        let color32 = Color32::from_rgb(
            (rgb[0] * 255.0).round() as u8,
            (rgb[1] * 255.0).round() as u8,
            (rgb[2] * 255.0).round() as u8,
        );
        // Use egui's built-in to_hex() but trim alpha to get 6-digit format
        let hex_with_alpha = color32.to_hex();
        format!("#{}", &hex_with_alpha[1..7]) // Remove # and alpha part
    }

    /// Convert hex string to RGB float array using egui's Color32
    fn hex_to_rgb(hex: &str) -> Option<[f32; 3]> {
        if let Ok(color32) = Color32::from_hex(hex) {
            Some([
                color32.r() as f32 / 255.0,
                color32.g() as f32 / 255.0,
                color32.b() as f32 / 255.0,
            ])
        } else {
            None
        }
    }

    /// Validate hex color format using egui's parser
    fn is_valid_hex_color(hex: &str) -> bool {
        Color32::from_hex(hex).is_ok()
    }
}

impl eframe::App for EquationProcessorApp {
    /// Called each frame to update application logic and draw the UI.
    ///
    /// This method:
    /// 1. Polls the background rendering channel for completion.
    /// 2. Handles file and directory dialog interactions.
    /// 3. Renders the main UI: selectors, options, process button,
    ///    spinner indicator, messages, and equations table.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Check for background render completion
        if let Some(rx) = &self.progress_rx {
            if rx.try_recv().is_ok() {
                self.processing = false;
                self.progress_rx = None;
                self.success_message = Some("Rendering complete!".into());
                ctx.request_repaint();
            }
        }

        // 2. Update file dialogs and load/validate input
        self.open_file_dialog.update(ctx);
        if let Some(path) = self.open_file_dialog.take_picked() {
            self.input_file = Some(path.clone());
            // Validate and parse by file type
            match detect_file_type(&path) {
                Filetype::Csv => {
                    self.equations = read_csv_file(&path).unwrap_or_default();
                    self.error_message = None;
                }
                Filetype::Markdown => {
                    let txt = std::fs::read_to_string(&path).unwrap_or_default();
                    self.equations = parse_markdown(&txt);
                    self.error_message = None;
                }
                Filetype::Unknown => {
                    self.equations.clear();
                    self.error_message = Some("Unsupported file type selected.".into());
                    self.success_message = None;
                }
            }
        }
        self.select_dir_dialog.update(ctx);
        if let Some(path) = self.select_dir_dialog.take_picked() {
            self.output_dir = Some(path);
        }

        // 3. Render UI components
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Equation Processor");
            ui.add_space(12.0);

            // Input file selector row
            ui.horizontal(|ui| {
                ui.label("Input file:");
                if ui.button("Browse…").clicked() {
                    self.open_file_dialog.pick_file();
                }
                if let Some(p) = &self.input_file {
                    ui.label(p.display().to_string());
                }
            });
            ui.add_space(8.0);

            // Output directory selector row
            ui.horizontal(|ui| {
                ui.label("Output dir:");
                if ui.button("Select…").clicked() {
                    self.select_dir_dialog.pick_directory();
                }
                if let Some(d) = &self.output_dir {
                    ui.label(d.display().to_string());
                }
            });
            ui.add_space(8.0);

            // Rendering options
            ui.horizontal(|ui| {
                ui.label("Font color:");
                
                
                // Color picker
                let picker_changed = ui.color_edit_button_rgb(&mut self.font_color).changed();
                
                // Hex text input
                let text_response = ui.add(
                    egui::TextEdit::singleline(&mut self.color_hex_input)
                        .desired_width(80.0)
                        .hint_text("#000000")
                );
                
                // Synchronize color picker -> hex input
                if picker_changed {
                    self.color_hex_input = Self::rgb_to_hex(self.font_color);
                }
                
                // Synchronize hex input -> color picker
                if text_response.changed() {
                    if Self::is_valid_hex_color(&self.color_hex_input) {
                        if let Some(rgb) = Self::hex_to_rgb(&self.color_hex_input) {
                            self.font_color = rgb;
                        }
                    }
                }
                
                // Show validation indicator
                if !Self::is_valid_hex_color(&self.color_hex_input) && !self.color_hex_input.is_empty() {
                    ui.colored_label(Color32::RED, "Invalid hex color");
                }
                
                ui.checkbox(&mut self.delete_intermediates, "Delete intermediates");
            });
            ui.add_space(12.0);

            // Process button and optional spinner indicator
            ui.horizontal(|ui| {
                let btn = ui.add_enabled(!self.processing, egui::Button::new("Process"));
                if btn.clicked() {
                    self.error_message = None;
                    self.success_message = None;
                    if self.equations.is_empty() {
                        self.error_message = Some("No equations loaded.".into());
                    } else if self.output_dir.is_none() {
                        self.error_message = Some("Select an output directory.".into());
                    } else {
                        // Spawn background render thread
                        let eqs = std::mem::take(&mut self.equations);
                        let out = self.output_dir.clone().unwrap();
                        let del = self.delete_intermediates;
                        let hex = format!(
                            "#{:02X}{:02X}{:02X}",
                            (self.font_color[0] * 255.0) as u8,
                            (self.font_color[1] * 255.0) as u8,
                            (self.font_color[2] * 255.0) as u8
                        );
                        let (tx, rx) = mpsc::channel();
                        self.progress_rx = Some(rx);
                        self.processing = true;
                        thread::spawn(move || {
                            for eq in eqs {
                                let _ = eq.render(&out, &hex, del);
                            }
                            let _ = tx.send(());
                        });
                    }
                }
                if self.processing {
                    ui.add(Spinner::new().size(16.0));
                    ui.label(" Rendering…");
                }
            });
            ui.add_space(12.0);

            // Display error or success messages
            if let Some(err) = &self.error_message {
                ui.colored_label(Color32::RED, err);
                ui.add_space(8.0);
            }
            if let Some(msg) = &self.success_message {
                ui.colored_label(Color32::from_rgb(0, 100, 0), msg);
                ui.add_space(8.0);
            }

            ui.separator();
            ui.add_space(8.0);

            // Equations table
            if !self.equations.is_empty() {
                // Select All/None buttons
                ui.horizontal(|ui| {
                    if ui.button("Select All").clicked() {
                        for eq in &mut self.equations {
                            eq.active = true;
                        }
                    }
                    if ui.button("Select None").clicked() {
                        for eq in &mut self.equations {
                            eq.active = false;
                        }
                    }
                });
                ui.add_space(8.0);
                ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::remainder().clip(true))
                        .header(24.0, |mut h| {
                            h.col(|ui| {
                                ui.heading("Active");
                            });
                            h.col(|ui| {
                                ui.heading("Name");
                            });
                            h.col(|ui| {
                                ui.heading("Equation");
                            });
                        })
                        .body(|mut b| {
                            for eq in &mut self.equations {
                                b.row(24.0, |mut r| {
                                    r.col(|ui| {
                                        ui.checkbox(&mut eq.active, "");
                                    });
                                    r.col(|ui| {
                                        ui.label(&eq.name);
                                    });
                                    r.col(|ui| {
                                        ui.label(&eq.body);
                                    });
                                });
                            }
                        });
                });
            }
        });
    }
}

/// Launch the Equation Processor GUI, reporting failures.
///
/// Attempts to open a native window sized 700×700 px and runs the eframe loop.
pub fn launch_gui() {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([700.0, 700.0]),
        ..Default::default()
    };
    if let Err(err) = eframe::run_native(
        "Equation Processor",
        options,
        Box::new(|cc| Ok(Box::new(EquationProcessorApp::new(cc)))),
    ) {
        eprintln!("Failed to launch GUI: {err}");
    }
}
