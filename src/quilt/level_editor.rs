mod mapdata;
mod endata;

use std::{fs, path::PathBuf};
use egui::{self, Button};
use gfarch::gfarch;
use mapdata::Mapdata;
// use endata::Endata;
use rfd::FileDialog;
use anyhow::Result;

use super::common::Camera;


#[derive(Default)]
pub struct LevelEditor {
    file_open: bool,
    file_path: PathBuf,
    archive_contents: Vec<gfarch::FileContents>,
    selected_file_index: usize,
    selected_pair_index: usize,
    current_mapdata: Mapdata,
    // current_endata: Endata,
    camera: Camera,
}

impl LevelEditor {
    fn set_pair(&mut self, enbin_index: usize) {
        // each enbin goes with a corresponding mapbin
        // though both will be rendered at the same time,
        // they can't be edited at the same time,
        // for the sake of ease of use.


        self.selected_pair_index = enbin_index;
    }

    fn update_level_data(&mut self) {
        println!("endata not implemented yet");
        // self.current_endata = Endata::from_data(
        //     &self.archive_contents[self.selected_pair_index].contents
        // );

        self.current_mapdata = Mapdata::from_data(
            &self.archive_contents[self.selected_pair_index + 1].contents
        );
    }

    fn open_file(&mut self) -> Result<()> {
        if let Some(path) = FileDialog::new()
        .add_filter("Level archive", &["gfa"])
        .pick_file() {
            self.file_path = path;
            let data = fs::read(&self.file_path)?;
            self.archive_contents = gfarch::extract(&data)?;
            self.selected_file_index = 0;
            self.set_pair(0);

            self.update_level_data();

            self.file_open = true;
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
        egui::ComboBox::from_label("Selected file")
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
                self.update_level_data();
            }
        });


        // canvas
        ui.label("Canvas");
        egui::Frame::canvas(ui.style())
        .show(ui, |ui|{
            self.camera.update(ui.ctx());
            
            ui.label(format!("Camera: x {}, y {}, zoom {}", self.camera.position.x, self.camera.position.y, self.camera.zoom));
            let desired_size = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());

            // draw black
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

            /* rendering */

            for wall in self.current_mapdata.walls.iter() {
                let start = rect.min + 
                    self.camera.to_camera(wall.start.to_vec2());
                let end = rect.min + 
                    self.camera.to_camera(wall.end.to_vec2());

                painter.line_segment(
                    [start, end],
                    egui::Stroke::new(1.0, egui::Color32::WHITE)
                );
            }

            for gmk in self.current_mapdata.gimmicks.iter_mut() {
                let pos = gmk.position.to_point_2d();
                let screen_pos = rect.min.to_vec2() + self.camera.to_camera(pos.to_vec2());
                let square = egui::Rect::from_center_size(
                    screen_pos.to_pos2(),
                    egui::Vec2::new(2.0 * self.camera.zoom, 2.0 * self.camera.zoom)
                );

                let resp = ui.interact(
                    square,
                    egui::Id::new(gmk as *const _),
                    egui::Sense::click_and_drag()
                );

                let color = if gmk.is_selected {
                    egui::Color32::RED
                } else {
                    egui::Color32::LIGHT_GRAY
                };

                painter.rect_filled(square, 0.0, color);

                if resp.hovered() {
                    painter.text(
                        screen_pos.to_pos2() + egui::Vec2::new(10.0, 10.0),
                        egui::Align2::LEFT_CENTER,
                        &gmk.name,
                        egui::FontId::default(),
                        egui::Color32::WHITE
                    );
                }
                
                if resp.clicked() {
                    gmk.is_selected = true;
                } else if resp.dragged() {
                    // let world = self.camera.from_camera(resp.drag_delta());
                    
                    let world_delta = resp.drag_delta() / self.camera.zoom;

                    gmk.position.x += world_delta.x;
                    gmk.position.y -= world_delta.y;
                }

                if ui.ctx().input(|i| i.pointer.button_down(egui::PointerButton::Secondary)) {
                    if !square.contains(ui.input(|i| i.pointer.latest_pos().unwrap_or_default())) {
                        gmk.is_selected = false;
                    }
                }
            }
            /* end rendering */

            // other stuff...

            if response.dragged() {
                let delta = response.drag_delta();
                self.camera.pan(delta / self.camera.zoom);
            }

            // handle attributes
            if let Some(gmk) = self.current_mapdata.gimmicks.iter_mut()
                .find(|g| g.is_selected)
            {
                egui::Area::new(egui::Id::from("le_attribute_editor"))
                
                .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
                .show(ui.ctx(), |ui|{
                    egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui|{
                        ui.label("Edit gimmick attributes");
                        ui.horizontal(|ui|{
                            ui.label("Name");
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut gmk.name
                                ).char_limit(0x30)
                            );
                        }); 
                    });
                });
            }
        });
    }
}
