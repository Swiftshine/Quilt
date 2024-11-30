mod mapdata;
mod endata;
mod le_io;
mod le_util;
mod le_canvas;
mod le_object;


use std::{fs, path::PathBuf};
use egui::{self, Button};
use gfarch::gfarch;
use mapdata::*;
use endata::*;

use super::common::Camera;
use serde_json;




#[derive(PartialEq)]
// These are indices
enum ObjectIndex {
    // Walls,
    // LabeledWalls,
    CommonGimmick(usize), 
    Gimmick(usize),
    // Zones,
    // CourseInfo,
    Enemy(usize)
}

#[derive(PartialEq)]
enum ObjectType {
    // Walls,
    // LabeledWalls,
    CommonGimmick(String), 
    Gimmick,
    // Zones,
    // CourseInfo,
    // Enemies
}


#[derive(Default)]
pub struct LevelEditor {
    // i/o
    file_open: bool,
    file_path: PathBuf,

    // files
    archive_contents: Vec<gfarch::FileContents>,
    selected_file_index: usize,
    selected_enbin_index: Option<usize>,
    selected_mapbin_index: Option<usize>,
    current_mapdata: Mapdata,
    current_endata: Endata,

    // editor
    display_none: bool,
    camera: Camera,
    selected_object_indices: Vec<ObjectIndex>,
    current_add_object: Option<ObjectType>,
    object_data_json: serde_json::Value,
    is_object_data_valid: bool,

    // level contents
    show_walls: bool,
    show_labeled_walls: bool,
    show_common_gimmicks: bool,
    show_gimmicks: bool,
    show_paths: bool,
    show_zones: bool,
    show_course_info: bool,

    // ui
    show_object_context_menu: bool,
    common_gimmick_object_query: String,
}

impl LevelEditor {
    pub fn new() -> Self {
        Self {
            object_data_json: if let Ok(b) = fs::exists("res/objectdata.json") {
                if b {
                    let contents = fs::read_to_string("res/objectdata.json").unwrap_or_else(|_| String::new());
                    serde_json::from_str(&contents).expect("failed to parse json")
                } else {
                    Default::default()
                }
            } else {
                Default::default()
            },

            show_walls: true,
            show_labeled_walls: true,
            show_common_gimmicks: true,
            show_gimmicks: true,
            show_paths: true,
            show_zones: true,
            show_course_info: true,

            ..Default::default()
        }
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
                    let _ = self.save_file(false);
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, Button::new("Save as"))
                .clicked() {
                    let _ = self.save_file(true);
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

    

    fn object_context_menu(&mut self, ui: &mut egui::Ui) {
        egui::Area::new(egui::Id::from("le_object_context_menu"))
        .anchor(
            egui::Align2::RIGHT_BOTTOM,
            egui::Vec2::splat(1.0)
        )
        .show(ui.ctx(), |ui|{
            egui::Frame::popup(ui.style())
            .inner_margin(egui::Vec2::splat(8.0))
            .show(ui, |ui|{
                ui.label("Add object");

                ui.collapsing("Add Common Gimmick", |ui|{
                    ui.label("Search");
                    ui.text_edit_singleline(&mut self.common_gimmick_object_query);
                    egui::ScrollArea::vertical()
                    .id_salt("le_add_common_gimmick")
                    .max_height(150.0)
                    .show(ui, |ui|{
                        // iterate through common gimmick names
    
                        let data = self.object_data_json.get("common_gimmicks")
                        .expect("couldn't find 'common_gimmicks' inside objectdata.json")
                        .as_object().unwrap();
                        
                        for (hex, values) in data {
                            let name = values.get("name").and_then(|s| s.as_str()).unwrap();

                            
                            if name.to_lowercase()
                            .contains(&self.common_gimmick_object_query.to_lowercase()) {
                                let mut selected = false;

                                ui.selectable_value(&mut selected, true, name);

                                if selected {
                                    self.current_add_object = Some(ObjectType::CommonGimmick(hex.to_owned()))
                                }
                            }


                        }
                        

                    });
                });

                if ui.button("Add Gimmick").clicked() {
                    self.current_add_object = Some(ObjectType::Gimmick);
                }

            });
        });
    }

    fn handle_inputs(&mut self, ui: &mut egui::Ui, response: &egui::Response) {
        
        if ui.ctx().input(|i| i.pointer.secondary_clicked()) {
            self.show_object_context_menu = !self.show_object_context_menu;
        }
        
        if self.show_object_context_menu {
            self.object_context_menu(ui);
        }

        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            self.camera.pan(delta / self.camera.zoom);
        }


        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
        }
    }

    

    fn process_object_attributes(&mut self, ui: &mut egui::Ui) {
        if self.selected_object_indices.len() != 1 {
            return;
        }

        match self.selected_object_indices[0] {
            ObjectIndex::CommonGimmick(index) => {
                self.process_common_gimmick_attributes(ui, index);   
            }

            ObjectIndex::Gimmick(index) => {
                self.process_gimmick_attributes(ui, index);
            }

            ObjectIndex::Enemy(index) => {
                self.process_enemy_attributes(ui, index);
            }
        }
    }

    
}
