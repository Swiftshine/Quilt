mod mapdata;
mod endata;

use std::{fs, path::PathBuf};
use egui::{self, Button, Rect, Color32};
use gfarch::gfarch;
use mapdata::Mapdata;
use reqwest::blocking::Client;
// use endata::Endata;
use rfd::FileDialog;
use anyhow::{bail, Result};
const SQUARE_SIZE: f32 = 2.0;
use super::common::Camera;
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
enum EditMode {
    View,
    // Walls,
    // LabeledWalls,
    // CommonGimmicks,
    // Gimmicks,
    // Zones,
    // CourseInfo,
}

#[derive(PartialEq)]
// These are indices
enum SelectType {
    // Walls,
    // LabeledWalls,
    CommonGimmick(usize), 
    Gimmick(usize),
    // Zones,
    // CourseInfo,
}



impl Default for EditMode {
    fn default() -> Self {
        EditMode::View
    }
}

#[derive(Default)]
pub struct LevelEditor {
    file_open: bool,
    file_path: PathBuf,
    archive_contents: Vec<gfarch::FileContents>,
    selected_file_index: usize,
    selected_pair_index: usize,
    // edit_mode: EditMode,
    current_mapdata: Mapdata,
    display_none: bool,
    // current_endata: Endata,
    camera: Camera,
    selected_object_indices: Vec<SelectType>,
    object_data_json: serde_json::Value,
    is_data_valid: bool,
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
        
        self.current_mapdata = if self.archive_contents.len() > self.selected_pair_index + 1{
            Mapdata::from_data(
                &self.archive_contents[self.selected_pair_index + 1].contents
            )
        } else {
            Mapdata::default()
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

            ui.checkbox(&mut self.display_none, "Display 'NONE'?")
            .on_hover_text("Indicates whether or not to display entities with a name of 'NONE'.");
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

            self.update_walls(ui, rect);
            self.update_labeled_walls(ui, rect);
            self.update_common_gimmicks(ui, rect);
            self.update_gimmicks(ui, rect);
            self.update_paths(ui, rect);
            self.update_zones(ui, rect);
            self.update_course_info(ui, rect);

            /* end rendering */

            // other stuff...

            self.handle_inputs(ui, &response);

            // handle attributes
            
            let object_data_exists = if let Ok(b) = fs::exists("res/objectdata.json") {
                b || !self.is_data_valid
            } else {
                false
            };

            if object_data_exists {
                self.process_object_attributes(ui);
            }
        });
    }

    fn handle_inputs(&mut self, ui: &mut egui::Ui, response: &egui::Response) {
        if response.dragged() {
            let delta = response.drag_delta();
            self.camera.pan(delta / self.camera.zoom);
        }

        if ui.ctx().input(|i| i.pointer.button_down(egui::PointerButton::Secondary)) {
            if !self.selected_object_indices.is_empty() {

                if self.selected_object_indices.len() == 1 {
                    match self.selected_object_indices[0] {
                        SelectType::CommonGimmick(index) => {
                            self.current_mapdata.common_gimmicks[index].is_selected = false;
                        }

                        SelectType::Gimmick(index) => {
                            self.current_mapdata.gimmicks[index].is_selected = false;
                        }
                    }
                }

            }
            self.selected_object_indices.clear();
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
                self.selected_object_indices.push(SelectType::CommonGimmick(index));
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
                self.selected_object_indices.push(SelectType::Gimmick(index));
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

    fn process_object_attributes(&mut self, ui: &mut egui::Ui) {
        if self.selected_object_indices.len() != 1 {
            return;
        }

        match self.selected_object_indices[0] {
            SelectType::CommonGimmick(index) => {
                self.process_common_gimmick_attributes(ui, index);   
            }

            SelectType::Gimmick(index) => {
                self.process_gimmick_attributes(ui, index);
            }
        }
    }

    fn process_common_gimmick_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        let gmk = &mut self.current_mapdata.common_gimmicks[index];

        if !gmk.is_selected {
            gmk.is_selected = true;
        }

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

                let data = self.object_data_json.get("common_gimmicks").expect("couldn't find 'common_gimmicks' inside objectdata.json");

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
        let gmk = &mut self.current_mapdata.gimmicks[index];
            
        if !gmk.is_selected {
            gmk.is_selected = true;
        }

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
}
