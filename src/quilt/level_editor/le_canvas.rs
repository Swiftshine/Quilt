use super::{
    LevelEditor,
    ObjectType
};

use std::fs;
use super::mapdata::*;
use crate::quilt::common::*;
use egui;

impl LevelEditor {
    pub fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui|{
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

                    // check if the index is that of an enbin
                    self.selected_enbin_index = 
                    if self.archive_contents[index].filename.contains(".enbin") {
                        Some(index)    
                    } else {
                        None
                    };
                    
                    self.selected_mapbin_index =
                    if index % 2 == 0 &&
                    self.archive_contents[index + 1].filename.contains(".mapbin") {
                        Some(index + 1)
                    } else if self.archive_contents[index].filename.contains(".mapbin") {
                        Some(index)
                    } else {
                        None
                    };
                    
                    self.update_level_data();
                }
            });

    
            if ui.button("Update data")
            .on_hover_text("Updates 'objectdata.json' from the internet.")
            .clicked() {
                if let Ok(_) = self.update_object_data() {
                    // println!("Succeeded.");
                } else {
                    println!("Failed.");
                }
            }
    
            if ui.button("Refresh data")
            .on_hover_text("Refreshes data from the local copy of 'objectdata.json'.")
            .clicked() {
                self.refresh_object_data();
            }

            ui.checkbox(&mut self.display_none, "Display 'NONE'?")
            .on_hover_text("Indicates whether or not to display entities with a name of 'NONE'.");

            // if ui.button("Remove 'NONE'?")
            // .on_hover_text("Any entities with a name of 'NONE' will be removed.")
            // .clicked() {
            //     self.current_mapdata.gimmicks.retain(|gimmick| gimmick.name != "NONE");
            //     self.current_mapdata.paths.retain(|path| path.name != "NONE");
            //     self.current_mapdata.zones.retain(|zone| zone.name != "NONE");
            // }
        });

        // canvas
        ui.horizontal(|ui|{
            ui.label("Canvas");
            ui.add_space(3.0);
            ui.checkbox(&mut self.show_walls, "Show Walls");
            ui.checkbox(&mut self.show_labeled_walls, "Show Labeled Walls");
            ui.checkbox(&mut self.show_common_gimmicks, "Show Common Gimmicks");
            ui.checkbox(&mut self.show_gimmicks, "Show Gimmicks");
            ui.checkbox(&mut self.show_paths, "Show Paths");
            ui.checkbox(&mut self.show_zones, "Show Zones");
            ui.checkbox(&mut self.show_course_info, "Show Course Info");
        });

        egui::Frame::canvas(ui.style())
        .show(ui, |ui|{
            
            ui.label(format!("Camera: x {}, y {}, zoom {}", self.camera.position.x, self.camera.position.y, self.camera.zoom));
            let desired_size = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());
            
            // update camera
            self.camera.update(ui.ctx(), &response);

            // draw black
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
            
            // object placement
            if let Some(object_type) = &self.current_add_object {
                if let Some(mut pointer_pos) = response.hover_pos() {
                    pointer_pos += rect.min.to_vec2();

                    painter.circle_filled(pointer_pos, 1.0, egui::Color32::GRAY);
                    let crosshair_size = 10.0;

                    // draw horizontal line
                    painter.line_segment(
                        [pointer_pos - egui::vec2(crosshair_size, 0.0), pointer_pos + egui::vec2(crosshair_size, 0.0)],
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );
                    
                    // draw vertical line
                    painter.line_segment(
                        [pointer_pos - egui::vec2(0.0, crosshair_size), pointer_pos + egui::vec2(0.0, crosshair_size)],
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );
                }

                if response.hovered() && ui.ctx().input(|i| i.pointer.any_released()) {
                    if let Some(pointer_pos) = response.hover_pos() {

                        match object_type {
                            ObjectType::Gimmick => {
                                let mut gmk = Gimmick::default();
                                
                                // the position gets put somewhere
                                // below the mouse. not sure how to fix it
                                let pos = self.camera.convert_from_camera(pointer_pos.to_vec2()).to_pos2();

                                gmk.position = Point2D::from_pos2(pos).to_point_3d();
                                gmk.name = String::from("NEW");
                                self.current_mapdata.gimmicks.push(gmk);
                            }

                            ObjectType::CommonGimmick(hex) => {
                                let mut gmk = CommonGimmick::default();

                                let pos = self.camera.convert_from_camera(pointer_pos.to_vec2()).to_pos2();
                                gmk.position = Point2D::from_pos2(pos).to_point_3d();
                                gmk.hex = hex.to_owned();
                                self.current_mapdata.common_gimmicks.push(gmk);

                                let hex_str = hex.to_owned();
                                if self.current_mapdata.common_gimmick_names.hex_names
                                .iter()
                                .position(|g| g.as_str() == &hex_str)
                                .is_none() {
                                    self.current_mapdata.common_gimmick_names.hex_names.push(hex.to_owned());
                                }
                            }

                            // _ => {}
                        }
                        self.current_add_object = None;
                    }
                }
            }

            /* rendering */

            if self.show_walls {
                self.update_walls(ui, rect);
            }

            if self.show_labeled_walls {
                self.update_labeled_walls(ui, rect);
            }

            if self.show_common_gimmicks {
                self.update_common_gimmicks(ui, rect);
            }
  
            if self.show_gimmicks {
                self.update_gimmicks(ui, rect);
            }

            if self.show_paths {
                self.update_paths(ui, rect);
            }

            if self.show_zones {
                self.update_zones(ui, rect);
            }

            if self.show_course_info {
                self.update_course_info(ui, rect);
            }

            self.update_enemies(ui, rect);

            /* end rendering */

            // other stuff...

            self.handle_inputs(ui, &response);

            // handle attributes
            
            let object_data_exists = if let Ok(b) = fs::exists("res/objectdata.json") {
                b || !self.is_object_data_valid
            } else {
                false
            };

            if object_data_exists {
                self.process_object_attributes(ui);
            }
        });
    }
}
