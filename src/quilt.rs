mod views;
mod level_editor;
mod gfarch_utility;
mod bgst_renderer;
mod bgst_editor;
mod common;
mod game;

use std::sync::Arc;

use eframe::{egui, NativeOptions};
use egui::IconData;
use views::QuiltView;
use level_editor::LevelEditor;
use bgst_editor::BGSTEditor;
use gfarch_utility::GfArchUtility;

pub struct QuiltApp {
    current_view: QuiltView,
    level_editor: LevelEditor,
    bgst_editor: BGSTEditor,
    gfarch_utility: GfArchUtility,
}

impl QuiltApp {
    /// Constructs a new application.
    fn new() -> Self {
        Self {
            current_view: QuiltView::Home,
            level_editor: LevelEditor::new(),
            bgst_editor: BGSTEditor::new(),
            gfarch_utility: GfArchUtility::new(),
        }
    }

    /// Starts the application.
    pub fn run() -> Result<(), eframe::Error> {
        let mut options = NativeOptions::default();

        options.viewport.icon = Some(
            Arc::new(
                IconData {
                    rgba: {
                        let icon = include_bytes!("../assets/icon.png");
                        let image = image::load_from_memory(icon)
                            .expect("Failed to open icon path")
                            .into_rgba8();

                        image.into_raw()
                    },

                    width: 48,
                    height: 48
                }
            )
        );


        eframe::run_native(
            "Quilt",
            options,
            Box::new(|_cc| {
                // QuiltApp::setup_fonts(&cc.egui_ctx);
                // egui_extras::install_image_loaders(&cc.egui_ctx);
                Ok(Box::<QuiltApp>::from( QuiltApp::new() ))
            })
        )
    }

    
    // fn setup_fonts(ctx: &egui::Context) {
    //     let mut fonts = egui::FontDefinitions::default();

    //     fonts.font_data.insert(
    //         String::from("noto_sans_jp"),
    //         egui::FontData::from_static(
    //             include_bytes!("../assets/font/NotoSans-jp.ttf")
    //         )
    //     );

    //     fonts.families.entry(egui::FontFamily::Proportional)
    //     .or_default()
    //     .insert(0, String::from("noto_sans_jp"));

    //     ctx.set_fonts(fonts);
    // }
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
                    ui.selectable_value(&mut self.current_view, QuiltView::BGSTEditor, "BGST Editor");
                    ui.selectable_value(&mut self.current_view, QuiltView::GfArchUtility, "GfArch Utility");
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

                QuiltView::BGSTEditor => {
                    self.bgst_editor.show_ui(ui);
                }

                QuiltView::GfArchUtility => {
                    self.gfarch_utility.show_ui(ui);
                }
            }
        });
    }
}
