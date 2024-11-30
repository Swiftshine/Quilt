use egui::{
    self,
    Color32,
    Rect
};

use super::{
    LevelEditor,
    ObjectIndex
};

use super::{
    color_string_to_label,
    label_to_color_string,
    enemy_id_to_name,
    ENEMY_LIST,
};

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

const SQUARE_SIZE: f32 = 2.0;


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

impl LevelEditor {
    /* object rendering */
    pub fn update_walls(&mut self, ui: &mut egui::Ui, rect: Rect) {
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

    pub fn update_labeled_walls(&mut self, ui: &mut egui::Ui, rect: Rect) {
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

    pub fn update_common_gimmicks(&mut self, ui: &mut egui::Ui, rect: Rect) {
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

    pub fn update_gimmicks(&mut self, ui: &mut egui::Ui, rect: Rect) {
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

    pub fn update_paths(&mut self, ui: &mut egui::Ui, rect: Rect) {
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

    pub fn update_zones(&mut self, ui: &mut egui::Ui, rect: Rect) {
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

    pub fn update_course_info(&mut self, ui: &mut egui::Ui, rect: Rect) {
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

    pub fn update_enemies(&mut self, ui: &mut egui::Ui, rect: Rect) {
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


    /* object attributes */
    pub fn process_common_gimmick_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
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

    pub fn process_gimmick_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
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


    pub fn process_enemy_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
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
