use super::bgst_renderer::BGSTRenderer;
use anyhow::{Result, bail};
use std::{fs, path::PathBuf};

mod be_canvas;

#[derive(Eq, PartialEq, Clone)]
enum TileSelection {
    Entry(usize),      // BGSTEntry index
    Empty((u32, u32)), // Tile coordinates (y, x)
}

#[derive(Default)]
pub struct BGSTEditor {
    bgst_renderer: BGSTRenderer,
    selected_layer: i16,
    file_path: Option<PathBuf>,
    selected_tile: Option<TileSelection>,
}

impl BGSTEditor {
    pub fn new() -> Self {
        Self {
            bgst_renderer: BGSTRenderer::new(),
            ..Default::default()
        }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("be_top_panel").show(ui.ctx(), |ui| {
            egui::menu::bar(ui, |ui| {
                // file submenu
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Ok(p) = self.bgst_renderer.open_file(ui) {
                            self.file_path = Some(p);
                            self.selected_tile = None;
                        }

                        ui.close_menu();
                    }

                    if ui
                        .add_enabled(
                            self.bgst_renderer.bgst_file.is_some(),
                            egui::Button::new("Save"),
                        )
                        .clicked()
                    {
                        // save file
                        let _ = self.save_file(false);
                        ui.close_menu();
                    }

                    if ui
                        .add_enabled(
                            self.bgst_renderer.bgst_file.is_some(),
                            egui::Button::new("Save as"),
                        )
                        .clicked()
                    {
                        // save file as
                        let _ = self.save_file(true);
                        ui.close_menu();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            if self.bgst_renderer.bgst_file.is_some() {
                self.render_contents(ui);
                self.handle_selected_tile(ui);
            }
        });
    }

    pub fn save_file(&mut self, save_as: bool) -> Result<()> {
        if save_as {
            match rfd::FileDialog::new()
                .add_filter("BGST file", &["bgst3"])
                .save_file()
            {
                Some(p) => self.file_path = Some(p),

                None => {
                    bail!("User exited.");
                }
            }
        }

        let bytes = self.bgst_renderer.bgst_file.as_ref().unwrap().get_bytes();

        fs::write(self.file_path.as_ref().unwrap(), bytes)?;

        Ok(())
    }
}
