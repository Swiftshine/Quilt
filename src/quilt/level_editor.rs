use std::{fs, path::PathBuf};

use egui::{self, Button};
use gfarch::gfarch;
use rfd::FileDialog;
use anyhow::Result;

#[derive(Default)]
pub struct LevelEditor {
    file_open: bool,
    file_path: PathBuf,
    archive_contents: Vec<gfarch::FileContents>,
    selected_file_index: usize,
    selected_pair_index: usize,
}

impl LevelEditor {
    
}

impl LevelEditor {
    fn set_pair(&mut self, enbin_index: usize) {
        // each enbin goes with a corresponding mapbin
        // though both will be rendered at the same time,
        // they can't be edited at the same time,
        // for the sake of ease of use.


        self.selected_pair_index = enbin_index;
    }

    fn open_file(&mut self) -> Result<()> {
        
        if let Some(path) = FileDialog::new()
        .add_filter("Level archive", &["gfa"])
        .pick_file() {
            self.file_path = path;
            let data = fs::read(&self.file_path)?;
            self.archive_contents = gfarch::extract(&data)?;
            self.file_open = true;
            self.selected_file_index = 0;
            self.set_pair(0);
        }

        Ok(())
    }

    fn save_file(&mut self, _save_as: bool) {
        todo!()
    }


    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("le_top_panel")
        .show(ui.ctx(), |ui|{
            egui::menu::bar(ui, |ui|{
                if ui.button("Open").clicked() {
                    let _ = self.open_file();
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, Button::new("Save"))
                .clicked() {
                    self.save_file(false);
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, Button::new("Save as"))
                .clicked() {
                    self.save_file(true);
                    ui.close_menu();
                }
            });
        });


        egui::CentralPanel::default().show(ui.ctx(), |ui|{
            if self.file_open {
                self.show_editor_ui(ui);
            }
        });

    }

    fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Selected files")
        .selected_text(
            &self.archive_contents[self.selected_file_index].filename
        ).show_ui(ui, |ui|{
            let mut index = self.selected_file_index;
            for i in 0..self.archive_contents.len() {
                ui.selectable_value(
                    &mut index,
                    i,
                    &self.archive_contents[i].filename
                );
            }

            if self.selected_file_index != index {
                self.selected_file_index = index;
                // the pairs will always be even because they share the
                // same index as that of the mapbin
                self.selected_pair_index = index - (index % 2);
            }
        });
    }
}
