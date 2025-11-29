// use super::{
//     le_object, EditMode, LevelEditor, ObjectType
// };
use super::{
    EditMode, LevelEditor, ObjectType
};

use crate::quilt::game::{
    endata::*,
    mapdata::*
};

use std::fs;

use crate::quilt::common::*;

impl LevelEditor {
    pub fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui|{
            egui::ComboBox::from_label("Selected file")
            .selected_text(
                &self.archive_contents[self.selected_file_index].0
            ).show_ui(ui, |ui|{
                let mut index = self.selected_file_index;
                for i in 0..self.archive_contents.len() {
                    ui.selectable_value(
                        &mut index,
                        i,
                        &self.archive_contents[i].0
                    );
                }
    
                if self.selected_file_index != index {
                    self.selected_file_index = index;

                    // check if the index is that of an enbin
                    self.selected_enbin_index = 
                    if self.archive_contents[index].0.contains(".enbin") {
                        Some(index)    
                    } else {
                        None
                    };
                    
                    self.selected_mapbin_index =
                    if index.is_multiple_of(2) &&
                    self.archive_contents[index + 1].0.contains(".mapbin") {
                        Some(index + 1)
                    } else if self.archive_contents[index].0.contains(".mapbin") {
                        Some(index)
                    } else {
                        None
                    };
                    
                    self.update_level_data();
                }
            });

            ui.checkbox(&mut self.display_none, "Display 'NONE'?")
            .on_hover_text("Indicates whether or not to display entities with a name of 'NONE'.");

            if ui.button("Remove 'NONE'?")
            .on_hover_text("Any entities with a name of 'NONE' will be removed.")
            .clicked() {
                self.current_mapdata.gimmicks.retain(|gimmick| gimmick.name != "NONE");
                self.current_mapdata.paths.retain(|path| path.name != "NONE");
                self.current_mapdata.zones.retain(|zone| zone.name != "NONE");
            }

            ui.collapsing("Level information", |ui|{
                ui.label("Mapdata").highlight();
                ui.horizontal(|ui|{
                    ui.label("Unknown @ 0x0");
                    ui.add(egui::DragValue::new(&mut self.current_mapdata.unk_0).range(f32::MIN..=f32::MAX));
                })
            });
        });

        // canvas options
        ui.horizontal(|ui|{
            ui.label("Canvas Options");
            ui.separator();

            let targets = [
                (&mut self.wall_edit_mode, "Walls"),
                (&mut self.labeled_wall_edit_mode, "Labeled Walls"),
                (&mut self.common_gimmick_edit_mode, "Common Gimmicks"),
                (&mut self.gimmick_edit_mode, "Gimmicks"),
                (&mut self.path_edit_mode, "Paths"),
                (&mut self.zone_edit_mode, "Zones"),
                (&mut self.course_info_edit_mode, "Course Infos")
            ];

            for (mode, text) in targets {
                // egui's combo boxes has the text be on the right of the 
                // combo box. frankly, it looks bad when multiple options
                // are strung together horizontally, because English
                // is read from left-to-right. 
                // unfortunately, there's no option to edit this,
                // so this will be the solution instead

                ui.label(text);
                
                egui::ComboBox::from_id_salt(format!("le_edit_mode_change_{}", text))
                .selected_text(Self::edit_mode_to_string(*mode))
                .show_ui(ui, |ui|{
                    let edit_modes = [
                        EditMode::Hide,
                        EditMode::View,
                        EditMode::Edit
                    ];

                    for edit_mode in edit_modes {
                        ui.selectable_value(mode, edit_mode, Self::edit_mode_to_string(edit_mode));
                    }
                });
            }
        });

        // the actual canvas drawn
        egui::Frame::canvas(ui.style())
        .show(ui, |ui|{
            // ui.label(format!("Camera x: {}, y: {}, zoom: {}", self.camera.position.x, self.camera.position.y, self.camera.zoom));
            let desired_size = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

            // update camera
            self.camera.update(ui.ctx(), &response);

            // draw black
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
            
            // object placement
            self.process_object_addition(ui, &response, rect, painter);
            
            /* rendering */

            // bgst rendering
            // if self.render_bgst {
            //     self.bgst_renderer.zoom = self.camera.zoom;

            //     let bg_base_index = self.current_mapdata.gimmicks.iter()
            //     .position(|g| &g.name == "BG_BASE");
            
            //     if let Some(bg_base_index) = bg_base_index {
            //         let bg_base = &self.current_mapdata.gimmicks[bg_base_index];

            //         // ensure the bg starts at the bottom left of the gimmick's rendering square
            //         let pos = egui::Vec2::new(
            //             bg_base.position.x - le_object::SQUARE_SIZE / 2.0,
            //             bg_base.position.y
            //         );
                    
            //         let pos = rect.min + self.camera.to_camera(pos);

            //         self.bgst_renderer.le_render(ui, rect, pos.to_vec2());
            //     }
            // }

            if !matches!(self.wall_edit_mode, EditMode::Hide) {
                self.update_walls(ui, rect);
            }

            if !matches!(self.labeled_wall_edit_mode, EditMode::Hide) {
                self.update_labeled_walls(ui, rect);
            }

            if !matches!(self.common_gimmick_edit_mode, EditMode::Hide) {
                self.update_common_gimmicks(ui, rect);
            }
            
            if !matches!(self.gimmick_edit_mode, EditMode::Hide) {
                self.update_gimmicks(ui, rect);
            }

            if !matches!(self.path_edit_mode, EditMode::Hide) {
                self.update_paths(ui, rect);
            }

            if !matches!(self.zone_edit_mode, EditMode::Hide) {
                self.update_zones(ui, rect);
            }

            if !matches!(self.course_info_edit_mode, EditMode::Hide) {
                self.update_course_info(ui, rect);
            }

            self.update_enemies(ui, rect);
            
            /* end rendering */

            // other stuff...

            self.handle_inputs(ui, &response);

            // handle attributes
            
            let object_data_exists = if let Ok(b) = fs::exists("quilt_res/objectdata.json") {
                b || !self.is_object_data_valid
            } else {
                false
            };

            if object_data_exists {
                self.process_object_attributes(ui);
            }
        });        
    }
    
    fn process_object_addition(&mut self, ui: &mut egui::Ui, response: &egui::Response, rect: egui::Rect, painter: egui::Painter) {
        // remove object if user doesn't want it
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.current_add_object = None;
            return;
        }

        if let Some(object_type) = &self.current_add_object {
            if let Some(pointer_pos) = response.hover_pos() {
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
                if let Some(mut pointer_pos) = response.hover_pos() {
                    pointer_pos -= rect.min.to_vec2();
                    
                    match object_type {
                        ObjectType::Wall => {
                            let collision_type = String::from("NML");
                            let start = self.camera.convert_from_camera(pointer_pos.to_vec2());
                            let start = Point2D::from_vec2(start);
                            let end = Point2D {
                                x: start.x + 5.0,
                                y: start.y
                            };
                            
                            let wall = Wall {
                                collision_type,
                                start,
                                end,
                                ..Default::default()
                            };
                            
                            self.current_mapdata.walls.push(wall);
                        }
                        
                        ObjectType::LabeledWall => {
                            let collision_type = String::from("NML");
                            let start = self.camera.convert_from_camera(pointer_pos.to_vec2());
                            let start = Point2D::from_vec2(start);
                            let end = Point2D {
                                x: start.x + 5.0,
                                y: start.y
                            };
                            
                            let wall = LabeledWall {
                                collision_type,
                                start,
                                end,
                                ..Default::default()
                            };
                            
                            self.current_mapdata.labeled_walls.push(wall);
                        }
                        
                        ObjectType::CommonGimmick(hex) => {
                            let mut gmk = CommonGimmick::default();
                            
                            let pos = self.camera.convert_from_camera(pointer_pos.to_vec2()).to_pos2();
                            gmk.position = Point2D::from_pos2(pos).get_point3d();
                            gmk.hex = hex.to_owned();
                            self.current_mapdata.common_gimmicks.push(gmk);
                            
                            let hex_str = hex.to_owned();
                            if !self.current_mapdata.common_gimmick_names.hex_names
                            .iter().any(|g| g.as_str() == hex_str) {
                                self.current_mapdata.common_gimmick_names.hex_names.push(hex.to_owned());
                            }
                            
                            self.add_common_gimmick_texture(ui.ctx(), &hex_str);
                        }
                        
                        ObjectType::Gimmick => {
                            let mut gmk = Gimmick::default();
                            
                            let pos = self.camera.convert_from_camera(pointer_pos.to_vec2()).to_pos2();
                            
                            gmk.position = Point2D::from_pos2(pos).get_point3d();
                            gmk.name = String::from("NEW");
                            self.current_mapdata.gimmicks.push(gmk);
                        }
                        
                        ObjectType::Path => {
                            let mut path = Path::default();
                            
                            let first = self.camera.convert_from_camera(pointer_pos.to_vec2());
                            path.points.push(Point2D {
                                x: first.x,
                                y: first.y
                            });
                            
                            path.points.push(Point2D {
                                x: first.x + 5.0,
                                y: first.y,
                            });
                            
                            path.name = String::from("NEW");
                            self.current_mapdata.paths.push(path);
                        }
                        
                        ObjectType::Zone => {
                            let mut zone = Zone::default();
                            let start = self.camera.convert_from_camera(pointer_pos.to_vec2()).to_pos2();
                            let end = start + egui::Vec2::splat(5.0);
                            
                            zone.bounds_start = Point2D::from_pos2(start);
                            zone.bounds_end = Point2D::from_pos2(end);
                            zone.name = String::from("NEW");
                            self.current_mapdata.zones.push(zone);
                        }
                        
                        ObjectType::CourseInfo => {
                            let mut course_info = CourseInfo::default();
                            let pos = self.camera.convert_from_camera(pointer_pos.to_vec2());
                            course_info.position = Point2D::from_vec2(pos).get_point3d();
                            course_info.name = String::from("NEW");
                            self.current_mapdata.course_infos.push(course_info);
                        }
                        
                        ObjectType::Enemy => {
                            let mut enemy = Enemy::new();
                            let pos = self.camera.convert_from_camera(pointer_pos.to_vec2()).to_pos2();
                            enemy.position_1 = Point2D::from_pos2(pos).get_point3d();
                            self.current_endata.enemies.push(enemy);
                        }
                        
                        // _ => {}
                    }
                    self.current_add_object = None;
                }
            }
        }
    }
}
