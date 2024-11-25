mod views;
mod level_editor;
mod common;

use eframe::{egui, NativeOptions};
use views::QuiltView;
use level_editor::LevelEditor;

pub struct QuiltApp {
    current_view: QuiltView,
    level_editor: LevelEditor
}

impl QuiltApp {
    /// Constructs a new application.
    fn new() -> Self {
        Self {
            current_view: QuiltView::Home,
            level_editor: LevelEditor::default()
        }
    }

    /// Starts the application.
    pub fn run() -> Result<(), eframe::Error> {
        eframe::run_native(
            "Quilt",
            NativeOptions::default(),
            Box::new(|_cc| {
                Ok(Box::<QuiltApp>::from( QuiltApp::new() ))
            })
        )
    }
}

impl eframe::App for QuiltApp {
    /// Called when the UI needs to be updated.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("q_top_panel")
        .show(ctx, |ui|{
            egui::menu::bar(ui, |ui|{
                ui.horizontal(|ui|{
                    ui.selectable_value(&mut self.current_view, QuiltView::Home, "Quilt");
                    ui.selectable_value(&mut self.current_view, QuiltView::LevelEditor, "Level Editor");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui|{
            match self.current_view {
                QuiltView::Home => {
                    ui.centered_and_justified(|ui|{
                        ui.label("Welcome to Quilt.");
                    });
                }
    
                QuiltView::LevelEditor => {
                    self.level_editor.show_ui(ui);
                }
            }
        });
    }
}
