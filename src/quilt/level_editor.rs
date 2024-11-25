mod mapdata;
mod endata;

use std::{fs, path::PathBuf};
use egui::{self, Button};
use gfarch::gfarch;
use mapdata::Mapdata;
use reqwest::blocking::Client;
// use endata::Endata;
use rfd::FileDialog;
use anyhow::{bail, Result};
const SQUARE_SIZE: f32 = 2.0;
use super::common::Camera;
use serde_json::{self, Value};

#[derive(PartialEq)]
enum DataType {
    None,
    Int,
    Float,
    String, // with a limit of 64 characters
}

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
    selected_gimmick_indices: Vec<usize>,
    object_data_contents: String,
}

impl LevelEditor {
    pub fn new() -> Self {
        Self {
            object_data_contents: if let Ok(b) = fs::exists("res/objectdata.json") {
                if b {
                    fs::read_to_string("res/objectdata.json").unwrap_or_else(|_| String::new())
                } else {
                    String::new()
                }
            } else {
                String::new()
            },

            ..Default::default()
        }
    }

    fn update_object_data(&mut self) -> Result<()> {
        let client = Client::new();

        let response = client
        .get("https://raw.githubusercontent.com/Swiftshine/key-objectdb/refs/heads/main/objectdata.json")
        .send()?;

        if !response.status().is_success() {
            bail!("failed to update objectdata.json");
        }

        let content = response.text()?;

        if let Ok(b) = fs::exists("res/") {
            if !b {
                fs::create_dir("res")?;
            }

            fs::write("res/objectdata.json", &content)?;
            self.object_data_contents = content;
        } else {
            bail!("failed to write objectdata.json");
        }

        Ok(())
    }

    fn refresh_object_data(&mut self) {
        if let Ok(s) = fs::read_to_string("res/objectdata.json") {
            self.object_data_contents = s;
        }
    }

    fn set_pair(&mut self, enbin_index: usize) {
        // each enbin goes with a corresponding mapbin
        // though both will be rendered at the same time,
        // they can't be edited at the same time,
        // for the sake of ease of use.
        self.selected_pair_index = enbin_index;
        self.selected_gimmick_indices.clear();
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
                    // the pairs will always be even because they share the
                    // same index as that of the mapbin
                    self.selected_pair_index = index - (index % 2);
                    self.update_level_data();
                }
            });
    
            if ui.button("Update data")
            .on_hover_text("Updates 'objectdata.json' from the internet.")
            .clicked() {
                if let Ok(_) = self.update_object_data() {
                    println!("Succeeded.");
                } else {
                    println!("Failed.");
                }
            }
    
            if ui.button("Refresh data")
            .on_hover_text("Refreshes data from the local copy of 'objectdata.json'.")
            .clicked() {
                self.refresh_object_data();
            }
        });

        // canvas
        ui.label("Canvas");
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
            
            for (index, gmk) in self.current_mapdata.gimmicks.iter_mut().enumerate() {
                let pos = gmk.position.to_point_2d();
                let screen_pos = rect.min.to_vec2() + self.camera.to_camera(pos.to_vec2());
                let square = egui::Rect::from_center_size(

                    {
                        let pos = screen_pos.to_pos2();

                        egui::Pos2::new(
                            pos.x,
                            pos.y - SQUARE_SIZE * 2.0
                        )
                    },
                    egui::Vec2::splat(SQUARE_SIZE * self.camera.zoom)
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
                        screen_pos.to_pos2() + egui::Vec2::new(10.0, -10.0),
                        egui::Align2::LEFT_CENTER,
                        &gmk.name,
                        egui::FontId::default(),
                        egui::Color32::WHITE
                    );
                }
                
                if resp.clicked() {
                    self.selected_gimmick_indices.push(index);
                } else if resp.dragged() {   
                    let world_delta = resp.drag_delta() / self.camera.zoom;

                    gmk.position.x += world_delta.x;
                    gmk.position.y -= world_delta.y;
                }
            }

            /* end rendering */

            // other stuff...

            if response.dragged() {
                let delta = response.drag_delta();
                self.camera.pan(delta / self.camera.zoom);
            }

            if ui.ctx().input(|i| i.pointer.button_down(egui::PointerButton::Secondary)) {
                if !self.selected_gimmick_indices.is_empty() {
                    let index = self.selected_gimmick_indices[0];
                    self.current_mapdata.gimmicks[index].is_selected = false;
                    self.selected_gimmick_indices.clear();
                }
            }

            // handle attributes
            
            let object_data_exists = if let Ok(b) = fs::exists("res/objectdata.json") {
                b || !self.object_data_contents.is_empty()
            } else {
                false
            };

            if object_data_exists {
                if self.selected_gimmick_indices.len() == 1 {
                    self.process_object_attributes(ui);
                }
            }
        });
    }

    fn process_object_attributes(&mut self, ui: &mut egui::Ui) {
        // gimmicks

        let gmk = 
            &mut self.current_mapdata.gimmicks[self.selected_gimmick_indices[0]];
            
        if !gmk.is_selected {
            gmk.is_selected = true;
        }
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

                let json: Value = serde_json::from_str(&self.object_data_contents).expect("failed to parse json");
                if let Some(data) = json.get("gimmicks") {
                    if let Some(gmk_data) = data.get(&gmk.name) {

                        if let Some(desc) = gmk_data.get("description") {
                            let desc = desc.as_str().unwrap();
                            if !desc.is_empty() {
                                ui.label(desc);
                            }
                        }

                        if let Some(note) = gmk_data.get("note") {
                            let note = note.as_str().unwrap();
                            if !note.is_empty() {
                                ui.label("Note");
                                ui.label(note);
                            }
                        }

                        if let Some(params) = gmk_data.get("parameters").and_then(|p| p.as_object()) {
                            for (param_name, param_data) in params {
                                ui.collapsing(param_name, |ui|{
                                    if let Some(param_desc) = param_data.get("description").and_then(|d| d.as_str()) {
                                        if !param_desc.is_empty() {
                                            ui.label(param_desc);
                                        }
                                    }

                                    if let Some(param_note) = param_data.get("note").and_then(|n| n.as_str()) {
                                        if !param_note.is_empty() {
                                            ui.label(format!("Note: {param_note}"));
                                        }
                                    }

                                    let slot = param_data.get("slot").and_then(|s| s.as_u64()).unwrap() as usize;

                                    let data_type = match param_data.get("data_type").and_then(|t| t.as_str()).unwrap() {
                                        "int" => DataType::Int,
                                        "float" => DataType::Float,
                                        "string" => DataType::String,

                                        _ => DataType::None
                                    };

                                    match data_type {
                                        DataType::Int => {
                                            ui.add(
                                                egui::DragValue::new(&mut gmk.params.int_params[slot])
                                                .speed(1)
                                                .range(i32::MIN..=i32::MAX)
                                            );
                                        }
                                        
                                        DataType::Float => {
                                            ui.add(
                                                egui::DragValue::new(&mut gmk.params.float_params[slot])
                                                .speed(1.0)
                                                .range(f32::MIN..=f32::MAX)
                                            );

                                        },

                                        DataType::String => {
                                            ui.add(
                                                egui::TextEdit::singleline(
                                                    &mut gmk.params.string_params[slot]
                                                ).char_limit(0x40)
                                            );
                                        },

                                        _ => {} // do nothing
                                    };
                                });
                            }
                        }
                    }
                }

                ui.collapsing("Raw data", |ui|{
                    ui.label("Edit fields regardless of documentation.");

                    ui.add_space(3.0);
                    ui.label("Int values");
                    ui.horizontal(|ui|{
                        for i in 0..3 {
                            ui.add(
                                egui::DragValue::new(&mut gmk.params.int_params[i])
                                .speed(1)
                                .range(i32::MIN..=i32::MAX)
                            );
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("Float values");
                    ui.horizontal(|ui|{
                        for i in 0..3 {
                            ui.add(
                                egui::DragValue::new(&mut gmk.params.float_params[i])
                                .speed(1)
                                .range(f32::MIN..=f32::MAX)
                            );
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("String values");
                    for i in 0..3 {
                        ui.add(
                            egui::TextEdit::singleline(
                                &mut gmk.params.string_params[i]
                            ).char_limit(0x40)
                        );
                    }
                });
            });
        });
    }
}
