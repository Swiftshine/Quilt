mod mapdata;
mod endata;

use std::{fs, path::PathBuf};
use egui::{self, Button, Color32, Rect};
use gfarch::gfarch;
use mapdata::*;
use reqwest::blocking::Client;
use endata::*;
use rfd::FileDialog;
use anyhow::{bail, Result};
const SQUARE_SIZE: f32 = 2.0;
use super::common::{Camera, Point2D};
use serde_json;

const GIMMICK_COLOR: Color32 = egui::Color32::from_rgb(
    0xF8, 0x33, 0x3C
);

const PATH_COLOR: Color32 = egui::Color32::from_rgb(
    0x44, 0xAF, 0x69
);

const COMMON_GIMMICK_COLOR: Color32 = egui::Color32::from_rgb(
    0xFC, 0xAB, 0x10
);

const ZONE_COLOR: Color32 = egui::Color32::from_rgb(
    0x2B, 0x9E, 0xB3
);

#[derive(PartialEq)]
enum DataType {
    None,
    Int,
    Bool,
    Float,
    String, // with a limit of 64 characters
    DropdownInt,
    DropdownFloat,
    DropdownString, // with a limit of 64 characters
}

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

    fn deselect_all(&mut self) {
        for select_type in self.selected_object_indices.iter() {
            match select_type {
                ObjectIndex::CommonGimmick(index) => {
                    self.current_mapdata.common_gimmicks[*index].is_selected = false;
                }

                ObjectIndex::Gimmick(index) => {
                    self.current_mapdata.gimmicks[*index].is_selected = false;
                }

                ObjectIndex::Enemy(index) => {
                    self.current_endata.enemies[*index].is_selected = false;
                }
            }
        }
        self.selected_object_indices.clear();
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
            self.object_data_json = serde_json::from_str(&content).expect("failed to parse json");
        } else {
            bail!("failed to write objectdata.json");
        }

        Ok(())
    }

    fn get_translated_common_gimmick_name(json: &serde_json::Value, hex: &str) -> Option<String> {
        let common = json.get("common_gimmicks")?;
        let data = common.get(hex)?;
        let translation = data.get("name").and_then(|s| s.as_str())?;
        Some(translation.to_string())
    }

    fn refresh_object_data(&mut self) {
        if let Ok(s) = fs::read_to_string("res/objectdata.json") {
            self.object_data_json = serde_json::from_str(&s).expect("failed to read json");
        }
    }
    
    fn update_level_data(&mut self) {
        self.current_endata = if let Some(enbin_index) = self.selected_enbin_index {
            Endata::from_data(
                &self.archive_contents[enbin_index].contents
            )
        } else {
            Default::default()
        };

        self.current_mapdata = if let Some(mapbin_index) = self.selected_mapbin_index {
            Mapdata::from_data(
                &self.archive_contents[mapbin_index].contents
            )
        } else {
            Default::default()
        };
        
        self.selected_object_indices.clear();
    }

    fn open_file(&mut self) -> Result<()> {
        if let Some(path) = FileDialog::new()
        .add_filter("Level archive", &["gfa"])
        .pick_file() {
            self.file_path = path;
            let data = fs::read(&self.file_path)?;
            self.archive_contents = gfarch::extract(&data)?;

            if self.archive_contents.len() == 0 {
                bail!("archive is invalid");
            }


            self.selected_enbin_index = 
            if self.archive_contents[0].filename.contains(".enbin") {
                Some(0)
            } else {
                None
            };


            
            self.selected_mapbin_index = if self.archive_contents.len() > 1
                && self.archive_contents[1].filename.contains(".mapbin")
            {
                Some(1)
            } else {
                None
            };

            self.selected_file_index = 0;

            self.update_level_data();

            self.file_open = true;
        }

        Ok(())
    }

    fn save_file(&mut self, save_as: bool) -> Result<()> {
        let path = if !save_as {
            self.file_path.clone()
        } else {
            match rfd::FileDialog::new()
            .add_filter("Level archive", &["gfa"])
            .save_file() {
                Some(p) => p,
                None => {
                    bail!("User exited.");
                }
            }
        };

        self.file_path = path;

        // enbin
        
        if let Some(index) = self.selected_enbin_index {
            self.archive_contents[index].contents = self.current_endata.to_bytes();
        }

        // mapbin
        if let Some(index) = self.selected_mapbin_index {
            self.archive_contents[index].contents = self.current_mapdata.to_bytes();
        }

        let archive = gfarch::pack_from_files(
            &self.archive_contents,
            gfarch::Version::V3,
            gfarch::CompressionType::BPE,
            gfarch::GFCPOffset::Default
        );

        fs::write(&self.file_path, archive)?;
        Ok(())
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

        if ui.ctx().input(|i| i.pointer.button_down(egui::PointerButton::Secondary)) {
            self.deselect_all();
        }
    }

    fn update_walls(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
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
    }

    fn update_labeled_walls(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        for wall in self.current_mapdata.labeled_walls.iter() {
            let start = rect.min + 
                self.camera.to_camera(wall.start.to_vec2());
            let end = rect.min + 
                self.camera.to_camera(wall.end.to_vec2());

            painter.line_segment(
                [start, end],
                egui::Stroke::new(1.0, egui::Color32::LIGHT_RED)
            );
        }
    }

    fn update_common_gimmicks(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        for (index, gmk) in self.current_mapdata.common_gimmicks.iter_mut().enumerate() {
            if &gmk.hex == "NONE" && !self.display_none {
                continue;
            }

            let pos = gmk.position.to_point_2d();
            let screen_pos = rect.min.to_vec2() +
                self.camera.to_camera(pos.to_vec2());

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
                COMMON_GIMMICK_COLOR
            } else{
                egui::Color32::LIGHT_GRAY
            };

            painter.rect_stroke(
                square,
                0.0,
                egui::Stroke::new(1.0, color)
            );

            if resp.hovered() {
                painter.text(
                    screen_pos.to_pos2() + egui::Vec2::new(10.0, -10.0),
                    egui::Align2::LEFT_CENTER,
                    {
                        let hex_name = gmk.hex.clone();

                        if let Some(name) =
                            Self::get_translated_common_gimmick_name(&self.object_data_json, &hex_name)
                        {
                            name
                        } else {
                            hex_name
                        }
                    },
                    egui::FontId::default(),
                    egui::Color32::WHITE
                );
            }

            if resp.clicked() {
                self.selected_object_indices.push(ObjectIndex::CommonGimmick(index));
            } else if resp.dragged() {
                let world_delta = resp.drag_delta() / self.camera.zoom;

                gmk.position.x += world_delta.x;
                gmk.position.y -= world_delta.y;
            }
        }
    }

    fn update_gimmicks(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        for (index, gmk) in self.current_mapdata.gimmicks.iter_mut().enumerate() {
            if &gmk.name == "NONE" && !self.display_none {
                continue;
            }

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
                GIMMICK_COLOR
            } else {
                egui::Color32::LIGHT_GRAY
            };

            painter.rect_stroke(square, 0.0, egui::Stroke::new(1.0, color));

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
                self.selected_object_indices.push(ObjectIndex::Gimmick(index));
            } else if resp.dragged() {   
                let world_delta = resp.drag_delta() / self.camera.zoom;

                gmk.position.x += world_delta.x;
                gmk.position.y -= world_delta.y;
            }
        }
    }

    fn update_paths(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);

        for path in self.current_mapdata.paths.iter() {
            if &path.name == "NONE" && !self.display_none {
                continue;
            }

            for i in 0..path.points.len() - 1 {
                let start = rect.min + 
                    self.camera.to_camera(path.points[i].to_vec2());
                let end = rect.min + 
                    self.camera.to_camera(path.points[i + 1].to_vec2());

                painter.line_segment(
                    [start, end],
                    egui::Stroke::new(1.0, PATH_COLOR)
                );
            }
        }
    }

    fn update_zones(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);

        for zone in self.current_mapdata.zones.iter() {
            if &zone.name == "NONE" && !self.display_none {
                continue;
            }

            let min = rect.min +
                self.camera.to_camera(zone.bounds_min.to_vec2());

            let max = rect.min + 
                self.camera.to_camera(zone.bounds_max.to_vec2());

            painter.rect_stroke(
                Rect::from_points(&[min, max]),
                0.0,
                egui::Stroke::new(1.0, ZONE_COLOR)
            );
        }
    }

    fn update_course_info(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);

        for info in self.current_mapdata.course_infos.iter() {
            if &info.name == "NONE" && !self.display_none {
                continue;
            }

            let pos = rect.min +
                self.camera.to_camera(info.position.to_point_2d().to_vec2());

            painter.rect_stroke(
                Rect::from_center_size(pos, egui::Vec2::splat(SQUARE_SIZE * self.camera.zoom)),
                0.0,
                egui::Stroke::new(1.0, egui::Color32::LIGHT_YELLOW)
            );
        }
    }

    fn update_enemies(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        
        for (index, enemy) in self.current_endata.enemies.iter_mut().enumerate() {
            let pos = enemy.position_1.to_point_2d();
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
                egui::Id::new(enemy as *const _),
                egui::Sense::click_and_drag()
            );

            let color = egui::Color32::from_rgb(
                0xE3, 0x96, 0xDF
            );

            painter.rect_stroke(square, 0.0, egui::Stroke::new(1.0, color));

            if resp.hovered() {
                painter.text(
                    screen_pos.to_pos2() + egui::Vec2::new(10.0, -10.0),
                    egui::Align2::LEFT_CENTER,
                    enemy_id_to_name(&enemy.name),
                    egui::FontId::default(),
                    egui::Color32::WHITE
                );
            }

            if resp.clicked() {
                self.selected_object_indices.push(ObjectIndex::Enemy(index));
            } else if resp.dragged() {   
                let world_delta = resp.drag_delta() / self.camera.zoom;

                enemy.position_1.x += world_delta.x;
                enemy.position_1.y -= world_delta.y;
            }
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

    fn process_common_gimmick_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i|{
            i.key_pressed(egui::Key::Delete)
        }) {
            self.current_mapdata.common_gimmicks.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i|{
            i.key_pressed(egui::Key::Escape)
        }) {
            self.deselect_all();
            return;
        }

        
        let gmk = &mut self.current_mapdata.common_gimmicks[index];

        gmk.is_selected = true;

        let (name, is_hex) = if let Some(n) = Self::get_translated_common_gimmick_name(
            &self.object_data_json, &gmk.hex
        ) {
            (n, false)
        } else {
            (gmk.hex.clone(), true)
        };


        egui::Area::new(egui::Id::from("le_common_gimmick_attribute_editor"))
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
        .show(ui.ctx(), |ui|{
            egui::Frame::popup(ui.style())
            .inner_margin(egui::Vec2::splat(8.0))
            .show(ui, |ui|{
                ui.label("Edit common gimmick attributes");

                if is_hex {
                    ui.label(format!("Hex: {name}"));
                } else {
                    ui.label(format!("Name: {name}"));
                }

                let data = self.object_data_json.get("common_gimmicks")
                .expect("couldn't find 'common_gimmicks' inside objectdata.json");

                if let Some(gmk_data) = data.get(&gmk.hex) {
                    if let Some(desc) = gmk_data.get("description").and_then(|d| d.as_str()) {
                        if !desc.is_empty() {
                            ui.label(desc);
                        }
                    }

                    if let Some(note) = gmk_data.get("note").and_then(|n| n.as_str()) {
                        if !note.is_empty() {
                            ui.label(format!("Note: {note}"));
                        }
                    }
                    
                    // regular parameters
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
                                    "bool" => DataType::Bool,
                                    "float" => DataType::Float,
                                    "string" => DataType::String,
                                    "dropdown_int" => DataType::DropdownInt,
                                    "dropdown_float" => DataType::DropdownFloat,
                                    "dropdown_string" => DataType::DropdownString,
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

                                    DataType::Bool => {
                                        let mut bool_value = gmk.params.int_params[slot] != 0;
                                        if ui.checkbox(&mut bool_value, "Value").changed() {
                                            gmk.params.int_params[slot] = if bool_value { 1 } else { 0 }
                                        }
                                    }
                                    
                                    DataType::Float => {
                                        ui.add(
                                            egui::DragValue::new(&mut gmk.params.float_params[slot])
                                            .speed(1.0)
                                            .range(f32::MIN..=f32::MAX)
                                        );
                                    }

                                    DataType::String => {
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                &mut gmk.params.string_params[slot]
                                            ).char_limit(0x40)
                                        );
                                    }

                                    DataType::DropdownInt => {
                                        let mut values: Vec<(String, i32)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_i64().unwrap() as i32
                                            ));

                                            if gmk.params.int_params[slot] == value.as_i64().unwrap() as i32 {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.int_params[slot],
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }

                                    DataType::DropdownFloat => {
                                        let mut values: Vec<(String, f32)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_f64().unwrap() as f32
                                            ));

                                            if gmk.params.float_params[slot] == value.as_f64().unwrap() as f32 {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.float_params[slot],
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }

                                    DataType::DropdownString => {
                                        let mut values: Vec<(String, String)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_str().unwrap().to_string()
                                            ));

                                            if gmk.params.string_params[slot] == value.as_str().unwrap().to_string() {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.string_params[slot],
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }


                                    DataType::None => {
                                        ui.label("The provided data type is invalid. You might want to check 'objectdata.json'.");
                                    }
                                };
                            });
                        }
                    }

                    // short parameters
                    if let Some(params) = gmk_data.get("short_parameters").and_then(|p| p.as_object()) {
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

                                let slot = param_data.get("slot").and_then(|s| s.as_u64()).unwrap_or(0) as usize;
                                let data_type = match param_data.get("data_type").and_then(|t| t.as_str()).unwrap() {
                                    "int" => DataType::Int,
                                    "bool" => DataType::Bool,
                                    "float" => DataType::Float,
                                    "string" => DataType::String,
                                    "dropdown_int" => DataType::DropdownInt,
                                    "dropdown_float" => DataType::DropdownFloat,
                                    "dropdown_string" => DataType::DropdownString,
                                    _ => DataType::None
                                };

                                match data_type {
                                    DataType::Int => {
                                        ui.add(
                                            egui::DragValue::new(&mut gmk.params.short_int_params[slot])
                                            .speed(1)
                                            .range(i32::MIN..=i32::MAX)
                                        );
                                    }

                                    DataType::Bool => {
                                        let mut bool_value = gmk.params.short_int_params[slot] != 0;
                                        if ui.checkbox(&mut bool_value, "Value").changed() {
                                            gmk.params.short_int_params[slot] = if bool_value { 1 } else { 0 }
                                        }
                                    }
                                    
                                    DataType::Float => {
                                        ui.add(
                                            egui::DragValue::new(&mut gmk.params.short_float_params[slot])
                                            .speed(1.0)
                                            .range(f32::MIN..=f32::MAX)
                                        );
                                    }

                                    DataType::String => {
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                &mut gmk.params.short_string_param
                                            ).char_limit(8)
                                        );
                                    }

                                    DataType::DropdownInt => {
                                        let mut values: Vec<(String, i32)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_i64().unwrap() as i32
                                            ));

                                            if gmk.params.short_int_params[slot] == value.as_i64().unwrap() as i32 {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.short_int_params[slot],
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }

                                    DataType::DropdownFloat => {
                                        let mut values: Vec<(String, f32)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_f64().unwrap() as f32
                                            ));

                                            if gmk.params.short_float_params[slot] == value.as_f64().unwrap() as f32 {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.short_float_params[slot],
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }

                                    DataType::DropdownString => {
                                        let mut values: Vec<(String, String)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_str().unwrap().to_string()
                                            ));

                                            if gmk.params.short_string_param == value.as_str().unwrap().to_string() {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.short_string_param,
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }


                                    DataType::None => {
                                        ui.label("The provided data type is invalid. You might want to check 'objectdata.json'.");
                                    }
                                };
                            });
                        }
                    }
                }

                ui.collapsing("Raw data", |ui|{
                    ui.label("Edit fields regardless of documentation.");

                    ui.add_space(3.0);
                    ui.label("Int values (common)");
                    ui.horizontal(|ui|{
                        for i in 0..2 {
                            ui.add(
                                egui::DragValue::new(&mut gmk.params.short_int_params[i])
                                .speed(1)
                                .range(i32::MIN..=i32::MAX)
                            );
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("Float values (common)");
                    ui.horizontal(|ui|{
                        for i in 0..2 {
                            ui.add(
                                egui::DragValue::new(&mut gmk.params.short_float_params[i])
                                .speed(1)
                                .range(f32::MIN..=f32::MAX)
                            );
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("String value (common)");
                    ui.add(
                        egui::TextEdit::singleline(&mut gmk.params.short_string_param)
                        .char_limit(8)
                    );

                    ui.add_space(3.0);
                    ui.label("Int values");
                    ui.horizontal(|ui|{
                        for i in 0..5 {
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
                        for i in 0..5 {
                            ui.add(
                                egui::DragValue::new(&mut gmk.params.float_params[i])
                                .speed(1)
                                .range(f32::MIN..=f32::MAX)
                            );
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("String values");
                    for i in 0..5 {
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

    fn process_gimmick_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i|{
            i.key_pressed(egui::Key::Delete)
        }) {
            self.current_mapdata.gimmicks.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i|{
            i.key_pressed(egui::Key::Escape)
        }) {
            self.deselect_all();
            return;
        }

        let gmk = &mut self.current_mapdata.gimmicks[index];

        gmk.is_selected = true;


        egui::Area::new(egui::Id::from("le_gimmick_attribute_editor"))
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

                let data = self.object_data_json.get("gimmicks").expect("couldn't find 'gimmicks' in objectdata.json");

                // paramter handling
                if let Some(gmk_data) = data.get(&gmk.name) {
                    if let Some(desc) = gmk_data.get("description").and_then(|d| d.as_str()) {
                        if !desc.is_empty() {
                            ui.label(desc);
                        }
                    }
                
                    if let Some(note) = gmk_data.get("note").and_then(|n| n.as_str()) {
                        if !note.is_empty() {
                            ui.label(format!("Note: {note}"));
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
                                    "bool" => DataType::Bool,
                                    "float" => DataType::Float,
                                    "string" => DataType::String,
                                    "dropdown_int" => DataType::DropdownInt,
                                    "dropdown_float" => DataType::DropdownFloat,
                                    "dropdown_string" => DataType::DropdownString,
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

                                    DataType::Bool => {
                                        let mut bool_value = gmk.params.int_params[slot] != 0;
                                        if ui.checkbox(&mut bool_value, "Value").changed() {
                                            gmk.params.int_params[slot] = if bool_value { 1 } else { 0 }
                                        }
                                    }
                                    
                                    DataType::Float => {
                                        ui.add(
                                            egui::DragValue::new(&mut gmk.params.float_params[slot])
                                            .speed(1.0)
                                            .range(f32::MIN..=f32::MAX)
                                        );
                                    }

                                    DataType::String => {
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                &mut gmk.params.string_params[slot]
                                            ).char_limit(0x40)
                                        );
                                    }

                                    DataType::DropdownInt => {
                                        let mut values: Vec<(String, i32)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_i64().unwrap() as i32
                                            ));

                                            if gmk.params.int_params[slot] == value.as_i64().unwrap() as i32 {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.int_params[slot],
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }

                                    DataType::DropdownFloat => {
                                        let mut values: Vec<(String, f32)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_f64().unwrap() as f32
                                            ));

                                            if gmk.params.float_params[slot] == value.as_f64().unwrap() as f32 {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.float_params[slot],
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }

                                    DataType::DropdownString => {
                                        let mut values: Vec<(String, String)> = Vec::new();
                                        let mut value_index = 0;
                                        for (value_name, value) in param_data.get("values")
                                        .and_then(|v| v.as_object()).unwrap() {
                                            values.push((
                                                value_name.clone(), value.as_str().unwrap().to_string()
                                            ));

                                            if gmk.params.string_params[slot] == value.as_str().unwrap().to_string() {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.string_params[slot],
                                                    value,
                                                    value_name
                                                );
                                            }
                                        });
                                    }


                                    DataType::None => {
                                        ui.label("The provided data type is invalid. You might want to check 'objectdata.json'.");
                                    }
                                };
                            });
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


    fn process_enemy_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i|{
            i.key_pressed(egui::Key::Delete)
        }) {
            self.current_endata.enemies.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i|{
            i.key_pressed(egui::Key::Escape)
        }) {
            self.deselect_all();
            return;
        }

        let enemy = &mut self.current_endata.enemies[index];

        enemy.is_selected = true;

        egui::Area::new(egui::Id::from("le_enemy_attribute_editor"))
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
        .show(ui.ctx(), |ui|{
            egui::Frame::popup(ui.style())
            .inner_margin(egui::Vec2::splat(8.0))
            .show(ui, |ui|{
                ui.label("Edit enemy attributes");

                egui::ComboBox::from_label("Enemy")
                .selected_text(enemy_id_to_name(&enemy.name))
                .show_ui(ui, |ui|{
                    for (id, name) in ENEMY_LIST {
                        ui.selectable_value(
                            &mut enemy.name,
                            String::from(id),
                            name
                        );
                    }
                });
 
                egui::ComboBox::from_label("Behavior")
                .selected_text(&enemy.behavior)
                .show_ui(ui, |ui|{
                    let behaviors = [
                        "STAND", "WALK1", "WALK2", "WALK_AREA",
                        "JUMP", "JUMP_LR", "FLOAT", "UP_DOWN",
                        "SLIDE", "SEARCH", "ATTACK1", "ATTACK2",
                        "ATTACK3", "READER", "FOLLOWING", "PURSUE",
                        "ESCAPE", "DEMO", "EVENT"
                    ];

                    for behavior in behaviors {
                        ui.selectable_value(
                            &mut enemy.behavior,
                            String::from(behavior),
                            behavior
                        );
                    }
                });

                ui.horizontal(|ui|{
                    ui.label("Path name");
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut enemy.path_name
                        ).char_limit(0x20)
                    );
                });

                egui::ComboBox::from_label("Bead type")
                .selected_text(&enemy.bead_type)
                .show_ui(ui, |ui|{
                    for i in 0..=11 {
                        let bead_type = format!("BEAD_KIND_{:02}", i);
                        ui.selectable_value(
                            &mut enemy.bead_type,
                            bead_type.clone(),
                            &bead_type,
                        );
                    }
                });

                egui::ComboBox::from_label("Bead color")
                .selected_text(
                    color_string_to_label(&enemy.bead_color)
                )
                .show_ui(ui, |ui|{
                
                    for color in [
                        "Red", "Orange", "Yellow", "Green",
                        "Blue", "Purple", "White", "Random"
                    ] {
                        ui.selectable_value(
                            &mut enemy.bead_color,
                            label_to_color_string(color),
                            color
                        );
                    }
                });

                egui::ComboBox::from_label("Direction")
                .selected_text(&enemy.direction)
                .show_ui(ui, |ui|{
                    let dirs = [
                        "RIGHT", "LEFT", "UP", "DOWN"
                    ];

                    for dir in dirs {   
                        ui.selectable_value(
                            &mut enemy.direction,
                            String::from(dir),
                            dir,
                        );
                    }
                });

                ui.horizontal(|ui|{
                    ui.label("Unknown @ 0x88");
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut enemy.unk_88
                        ).char_limit(8)
                    );
                });


                egui::ComboBox::from_label("Orientation")
                .selected_text(&enemy.orientation)
                .show_ui(ui, |ui|{
                    let orientations = [
                        "NONE", "FRONT", "BACK"
                    ];

                    for orientation in orientations {   
                        ui.selectable_value(
                            &mut enemy.orientation,
                            String::from(orientation),
                            orientation,
                        );
                    }
                });

                ui.collapsing("Parameters", |ui|{
                    for i in 0..7 {
                        ui.collapsing(format!("Set {}", i + 1), |ui|{
                            ui.label("Float values");
                            ui.horizontal(|ui|{
                                for j in 0..3 {
                                    ui.add(
                                        egui::DragValue::new(&mut enemy.params[i].float_params[j])
                                        .speed(1.0)
                                        .range(f32::MIN..=f32::MAX)
                                    );
                                }
                            });

                            ui.label("Int values");
                            ui.horizontal(|ui|{
                                for j in 0..3 {
                                    ui.add(
                                        egui::DragValue::new(&mut enemy.params[i].int_params[j])
                                        .speed(1.0)
                                        .range(f32::MIN..=f32::MAX)
                                    );
                                }
                            });
                        });
                    }
                });
            });
        });
    }
}
