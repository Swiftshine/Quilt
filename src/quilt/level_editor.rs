mod le_io;
mod le_util;
mod le_canvas;
mod le_object;


use std::{collections::HashMap, fs, path::PathBuf};
// use egui::{self, Button, DragValue, TextureHandle};
use egui::{self, Button, TextureHandle};
// use super::{bgst_renderer::BGSTRenderer, common::Camera};
use super::common::Camera;

use crate::quilt::game::{
    endata::*,
    mapdata::*,
};

#[derive(PartialEq)]
// These are indices
enum ObjectIndex {
    Wall(usize),
    LabeledWall(usize),
    CommonGimmick(usize), 
    Gimmick(usize),
    Path(usize),
    Zone(usize),
    CourseInfo(usize),
    Enemy(usize)
}

#[derive(PartialEq)]
enum ObjectType {
    Wall,
    LabeledWall,
    CommonGimmick(String), 
    Gimmick,
    Path,
    Zone,
    CourseInfo,
    Enemy
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum EditMode {
    Hide,
    #[default]
    View,
    Edit
}

#[derive(Default)]
pub struct LevelEditor {
    // i/o
    file_open: bool,
    file_path: PathBuf,

    // files
    archive_contents: Vec<(String, Vec<u8>)>,
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

    // editing modes
    wall_edit_mode: EditMode,
    labeled_wall_edit_mode: EditMode,
    common_gimmick_edit_mode: EditMode,
    gimmick_edit_mode: EditMode,
    path_edit_mode: EditMode,
    zone_edit_mode: EditMode,
    course_info_edit_mode: EditMode,

    // ui
    show_object_context_menu: bool,
    common_gimmick_object_query: String,

    // graphics
    object_textures: HashMap<String, TextureHandle>,
    render_bgst: bool,
    // bgst_renderer: BGSTRenderer
}

impl LevelEditor {
    pub fn new() -> Self {
        Self {
            object_data_json: if let Ok(b) = fs::exists("quilt_res/objectdata.json") {
                if b {
                    let contents = fs::read_to_string("quilt_res/objectdata.json").unwrap_or_else(|_| String::new());
                    serde_json::from_str(&contents).expect("failed to parse json")
                } else {
                    Default::default()
                }
            } else {
                Default::default()
            },

            common_gimmick_edit_mode: EditMode::Edit,
            gimmick_edit_mode: EditMode::Edit,
            zone_edit_mode: EditMode::Edit,
            course_info_edit_mode: EditMode::Edit,
            // bgst_renderer: BGSTRenderer::new(),

            ..Default::default()
        }
    }


    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("le_top_panel")
        .show(ui.ctx(), |ui|{
            egui::menu::bar(ui, |ui|{
                if ui.button("Open Archive").clicked() {
                    let _ = self.open_file(ui.ctx());
                    ui.close_menu();
                }
                
                if ui.button("Open Folder").clicked() {
                    let _ = self.open_folder(ui.ctx());
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, Button::new("Save Archive"))
                .clicked() {
                    let _ = self.save_file(false);
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, Button::new("Save Archive as"))
                .clicked() {
                    let _ = self.save_file(true);
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, Button::new("Save Folder"))
                .clicked() {
                    let _ = self.save_folder(false);
                    ui.close_menu();
                }

                if ui.add_enabled(self.file_open, Button::new("Save Folder as"))
                .clicked() {
                    let _ = self.save_folder(true);
                    ui.close_menu();
                }

                // if ui.add_enabled(self.file_open, Button::new("Open BGST"))
                // .clicked() {
                //     let _ = self.bgst_renderer.open_file(ui);
                //     ui.close_menu();
                // }

                // let bg_base_found = 
                // if self.file_open {
                //     // check mapdata
                //     self.current_mapdata.gimmicks.iter().any(|g| &g.name == "BG_BASE")
                // } else { 
                //     self.render_bgst = false;

                //     false 
                // };

                // let bgst_valid = self.bgst_renderer.bgst_file.is_some() && bg_base_found;

                // if bgst_valid {
                //     ui.checkbox(&mut self.render_bgst, "Display background?")
                //     .on_hover_text("This rendering is by no means perfect; it is a best estimate. Use the values on the right to change render settings.");

                //     ui.label("Tile size");
                //     ui.add(
                //         DragValue::new(&mut self.bgst_renderer.tile_size).speed(0.1)
                //     );


                //     ui.label("Tile X offset");
                //     ui.add(
                //         DragValue::new(&mut self.bgst_renderer.tile_offset.x).speed(0.1)
                //     );
                    
                //     ui.label("Tile Y offset");
                //     ui.add(
                //         DragValue::new(&mut self.bgst_renderer.tile_offset.y).speed(0.1)
                //     );
    
                    
                //     ui.label("Tile X scale");
                //     ui.add(
                //         DragValue::new(&mut self.bgst_renderer.tile_scale.x).speed(0.1)
                //     );
                    
                //     ui.label("Tile Y scale");
                //     ui.add(
                //         DragValue::new(&mut self.bgst_renderer.tile_scale.y).speed(0.1)
                //     );

                //     ui.label("Tile opacity");
                //     ui.add(
                //         DragValue::new(&mut self.bgst_renderer.opacity).speed(1).range(u8::MIN..=u8::MAX)
                //     );
                // }
                
                
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
                
                if ui.button("Add Wall").clicked() {
                    self.current_add_object = Some(ObjectType::Wall);
                }

                if ui.button("Add Labeled Wall").clicked() {
                    self.current_add_object = Some(ObjectType::LabeledWall);
                }

                ui.collapsing("Add Common Gimmick", |ui|{
                    ui.label("Search");
                    ui.text_edit_singleline(&mut self.common_gimmick_object_query);
                    egui::ScrollArea::vertical()
                    .id_salt("le_add_common_gimmick")
                    .max_height(150.0)
                    .show(ui, |ui|{
                        // iterate through common gimmick names
    
                        let data = self.object_data_json.get("common_gimmicks")
                        .expect("couldn't find 'common_gimmicks' in objectdata.json")
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

                if ui.button("Add Path").clicked() {
                    self.current_add_object = Some(ObjectType::Path);
                }

                if ui.button("Add Zone").clicked() {
                    self.current_add_object = Some(ObjectType::Zone);
                }

                if ui.button("Add Course Info").clicked() {
                    self.current_add_object = Some(ObjectType::CourseInfo);
                }

                if ui.button("Add Enemy").clicked() {
                    self.current_add_object = Some(ObjectType::Enemy);
                }
            });
        });
    }

    fn handle_inputs(&mut self, ui: &mut egui::Ui, response: &egui::Response) {
        if response.clicked_by(egui::PointerButton::Secondary) {
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
            ObjectIndex::Wall(index) => {
                self.process_wall_attributes(ui, index);
            }

            ObjectIndex::LabeledWall(index) => {
                self.process_labeled_wall_attributes(ui, index);
            }
            
            ObjectIndex::CommonGimmick(index) => {
                self.process_common_gimmick_attributes(ui, index);   
            }

            ObjectIndex::Gimmick(index) => {
                self.process_gimmick_attributes(ui, index);
            }

            ObjectIndex::Path(index) => {
                self.process_path_attributes(ui, index);
            }

            ObjectIndex::Zone(index) => {
                self.process_zone_attributes(ui, index);
            }

            ObjectIndex::CourseInfo(index) => {
                self.process_course_info_attributes(ui, index);
            }

            ObjectIndex::Enemy(index) => {
                self.process_enemy_attributes(ui, index);
            }
        }
    }

    
}
