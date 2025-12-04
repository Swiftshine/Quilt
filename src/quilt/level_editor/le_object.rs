use anyhow::{Context, Result};

use egui::{self, Color32, Rect};

use super::{COLLISION_TYPES, EditMode, LevelEditor, ObjectIndex, Params};

use super::{ENEMY_LIST, color_string_to_label, enemy_id_to_name, label_to_color_string};

// const WALL_COLOR: Color32 = egui::Color32::from_rgb(
//     0xF5, 0x8A, 0x07
// );

const GIMMICK_COLOR: Color32 = egui::Color32::from_rgb(0xF8, 0x33, 0x3C);

const PATH_COLOR: Color32 = egui::Color32::from_rgb(0x44, 0xAF, 0x69);

const COMMON_GIMMICK_COLOR: Color32 = egui::Color32::from_rgb(0xFC, 0xAB, 0x10);

const ZONE_COLOR: Color32 = egui::Color32::from_rgb(0x2B, 0x9E, 0xB3);

const COURSE_INFO_COLOR: Color32 = egui::Color32::from_rgb(0xEA, 0x8C, 0x55);

pub const SQUARE_SIZE: f32 = 2.0;
const CIRCLE_RADIUS: f32 = 0.1;

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

enum ByteOrder {
    Big,
    Little,
}

impl LevelEditor {
    /* object rendering */
    pub fn update_walls(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let painter = ui.painter_at(canvas_rect);
        for (index, wall) in self.current_mapdata.walls.iter_mut().enumerate() {
            let start = canvas_rect.min + self.camera.to_camera(wall.start.get_vec2());
            let end = canvas_rect.min + self.camera.to_camera(wall.end.get_vec2());

            let start_rect = egui::Rect::from_center_size(
                egui::Pos2::new(start.x, start.y - CIRCLE_RADIUS * 2.0),
                egui::Vec2::splat(CIRCLE_RADIUS * self.camera.zoom),
            );

            let end_rect = egui::Rect::from_center_size(
                egui::Pos2::new(end.x, end.y - CIRCLE_RADIUS * 2.0),
                egui::Vec2::splat(CIRCLE_RADIUS * self.camera.zoom),
            );

            let start_resp = ui.interact(
                canvas_rect.intersect(start_rect),
                egui::Id::new(&wall.start as *const _),
                egui::Sense::click_and_drag(),
            );

            let end_resp = ui.interact(
                canvas_rect.intersect(end_rect),
                egui::Id::new(&wall.end as *const _),
                egui::Sense::click_and_drag(),
            );

            let color = if wall.is_selected {
                egui::Color32::LIGHT_BLUE
            } else {
                egui::Color32::WHITE
            };

            painter.line_segment([start, end], egui::Stroke::new(1.0, egui::Color32::WHITE));

            if !matches!(self.wall_edit_mode, EditMode::Edit) {
                continue;
            }

            painter.circle_filled(start, CIRCLE_RADIUS * self.camera.zoom, color);
            painter.circle_filled(end, CIRCLE_RADIUS * self.camera.zoom, color);

            let mut clicked = false;
            let mut dragged = false;
            if start_resp.clicked() {
                clicked = true;
            } else if start_resp.dragged() {
                let world_delta = start_resp.drag_delta() / self.camera.zoom;
                wall.start.x += world_delta.x;
                wall.start.y -= world_delta.y;

                dragged = true;
            }

            if end_resp.clicked() {
                clicked = true;
            } else if end_resp.dragged() {
                let world_delta = end_resp.drag_delta() / self.camera.zoom;
                wall.end.x += world_delta.x;
                wall.end.y -= world_delta.y;

                dragged = true;
            }

            if clicked {
                self.selected_object_indices.push(ObjectIndex::Wall(index));
            }

            if dragged {
                wall.set_normalized_vector();
            }
        }
    }

    pub fn update_labeled_walls(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let painter = ui.painter_at(canvas_rect);
        for (index, wall) in self.current_mapdata.labeled_walls.iter_mut().enumerate() {
            let start = canvas_rect.min + self.camera.to_camera(wall.start.get_vec2());
            let end = canvas_rect.min + self.camera.to_camera(wall.end.get_vec2());

            let start_rect = egui::Rect::from_center_size(
                egui::Pos2::new(start.x, start.y - CIRCLE_RADIUS * 2.0),
                egui::Vec2::splat(CIRCLE_RADIUS * self.camera.zoom),
            );

            let end_rect = egui::Rect::from_center_size(
                egui::Pos2::new(end.x, end.y - CIRCLE_RADIUS * 2.0),
                egui::Vec2::splat(CIRCLE_RADIUS * self.camera.zoom),
            );

            let start_resp = ui.interact(
                canvas_rect.intersect(start_rect),
                egui::Id::new(&wall.start as *const _),
                egui::Sense::click_and_drag(),
            );

            let end_resp = ui.interact(
                canvas_rect.intersect(end_rect),
                egui::Id::new(&wall.end as *const _),
                egui::Sense::click_and_drag(),
            );

            let color = if wall.is_selected {
                egui::Color32::RED
            } else {
                egui::Color32::WHITE
            };

            painter.line_segment(
                [start, end],
                egui::Stroke::new(1.0, egui::Color32::LIGHT_RED),
            );

            if !matches!(self.labeled_wall_edit_mode, EditMode::Edit) {
                continue;
            }

            painter.circle_filled(start, CIRCLE_RADIUS * self.camera.zoom, color);
            painter.circle_filled(end, CIRCLE_RADIUS * self.camera.zoom, color);

            let mut clicked = false;
            let mut dragged = false;
            if start_resp.clicked() {
                clicked = true;
            } else if start_resp.dragged() {
                let world_delta = start_resp.drag_delta() / self.camera.zoom;
                wall.start.x += world_delta.x;
                wall.start.y -= world_delta.y;

                dragged = true;
            }

            if end_resp.clicked() {
                clicked = true;
            } else if end_resp.dragged() {
                let world_delta = end_resp.drag_delta() / self.camera.zoom;
                wall.end.x += world_delta.x;
                wall.end.y -= world_delta.y;

                dragged = true;
            }

            if clicked {
                self.selected_object_indices
                    .push(ObjectIndex::LabeledWall(index));
            }

            if dragged {
                wall.set_normalized_vector();
            }
        }
    }

    pub fn update_common_gimmicks(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) -> Result<()> {
        let painter = ui.painter_at(canvas_rect);
        for (index, gmk) in self.current_mapdata.common_gimmicks.iter_mut().enumerate() {
            if &gmk.hex == "NONE" && !self.display_none {
                continue;
            }

            let pos = gmk.position.get_point2d();
            let screen_pos = canvas_rect.min.to_vec2() + self.camera.to_camera(pos.get_vec2());

            let square = egui::Rect::from_center_size(
                {
                    let pos = screen_pos.to_pos2();

                    egui::Pos2::new(pos.x, pos.y - SQUARE_SIZE * 2.0)
                },
                egui::Vec2::splat(SQUARE_SIZE * self.camera.zoom),
            );

            let resp = ui.interact(
                canvas_rect.intersect(square),
                egui::Id::new(gmk as *const _),
                egui::Sense::click_and_drag(),
            );

            if let Some(texture) = self
                .object_textures
                .get(&format!("common_gimmick-{}", &gmk.hex))
            {
                if gmk.is_selected && matches!(self.common_gimmick_edit_mode, EditMode::Edit) {
                    painter.rect_filled(
                        square,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x10),
                    );
                }

                painter.image(
                    texture.id(),
                    square,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                    egui::Color32::WHITE,
                );
            } else {
                let color = if gmk.is_selected {
                    COMMON_GIMMICK_COLOR
                } else {
                    egui::Color32::LIGHT_GRAY
                };

                painter.rect_stroke(square, 0.0, egui::Stroke::new(1.0, color));
            }

            if !matches!(self.common_gimmick_edit_mode, EditMode::Edit) {
                continue;
            }

            if resp.hovered() {
                painter.text(
                    screen_pos.to_pos2() + egui::Vec2::new(10.0, -10.0),
                    egui::Align2::LEFT_CENTER,
                    {
                        let hex_name = gmk.hex.clone();

                        if let Some(name) = Self::get_translated_common_gimmick_name(
                            self.object_data_json
                                .as_ref()
                                .context("object_data_json is None")?,
                            &hex_name,
                        ) {
                            name
                        } else {
                            hex_name
                        }
                    },
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );
            }

            if resp.clicked() {
                self.selected_object_indices
                    .push(ObjectIndex::CommonGimmick(index));
            } else if resp.dragged() {
                let world_delta = resp.drag_delta() / self.camera.zoom;

                gmk.position.x += world_delta.x;
                gmk.position.y -= world_delta.y;
            }
        }

        Ok(())
    }

    pub fn update_gimmicks(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let painter = ui.painter_at(canvas_rect);
        for (index, gmk) in self.current_mapdata.gimmicks.iter_mut().enumerate() {
            if &gmk.name == "NONE" && !self.display_none {
                continue;
            }

            // add texture if not in current level's texture cache
            let key = format!("gimmick-{}", &gmk.name);
            if let std::collections::hash_map::Entry::Vacant(e) =
                self.object_textures.entry(key.clone())
                && let Ok(image_data) = Self::load_image_from_tex_folder("gimmick", &gmk.name)
            {
                let texture = ui
                    .ctx()
                    .load_texture(&key, image_data, egui::TextureOptions::LINEAR);

                e.insert(texture);
            }

            let pos = gmk.position.get_point2d();
            let screen_pos = canvas_rect.min.to_vec2() + self.camera.to_camera(pos.get_vec2());
            let square = egui::Rect::from_center_size(
                {
                    let pos = screen_pos.to_pos2();

                    egui::Pos2::new(pos.x, pos.y - SQUARE_SIZE * 2.0)
                },
                egui::Vec2::splat(SQUARE_SIZE * self.camera.zoom),
            );

            let resp = ui.interact(
                canvas_rect.intersect(square),
                egui::Id::new(gmk as *const _),
                egui::Sense::click_and_drag(),
            );

            if let Some(texture) = self.object_textures.get(&format!("gimmick-{}", &gmk.name)) {
                if gmk.is_selected && matches!(self.gimmick_edit_mode, EditMode::Edit) {
                    painter.rect_filled(
                        square,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x10),
                    );
                }

                painter.image(
                    texture.id(),
                    square,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                    egui::Color32::WHITE,
                );
            } else {
                let color = if gmk.is_selected {
                    GIMMICK_COLOR
                } else {
                    egui::Color32::LIGHT_GRAY
                };

                painter.rect_stroke(square, 0.0, egui::Stroke::new(1.0, color));
            }

            if !matches!(self.gimmick_edit_mode, EditMode::Edit) {
                continue;
            }

            if resp.hovered() {
                painter.text(
                    screen_pos.to_pos2() + egui::Vec2::new(10.0, -10.0),
                    egui::Align2::LEFT_CENTER,
                    &gmk.name,
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );
            }

            if resp.clicked() {
                self.selected_object_indices
                    .push(ObjectIndex::Gimmick(index));
            } else if resp.dragged() {
                let world_delta = resp.drag_delta() / self.camera.zoom;

                gmk.position.x += world_delta.x;
                gmk.position.y -= world_delta.y;
            }
        }
    }

    pub fn update_paths(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let painter = ui.painter_at(canvas_rect);

        for (index, path) in self.current_mapdata.paths.iter_mut().enumerate() {
            if &path.name == "NONE" && !self.display_none {
                continue;
            }

            for i in 0..path.points.len() - 1 {
                let start_pos = canvas_rect.min + self.camera.to_camera(path.points[i].get_vec2());
                let end_pos =
                    canvas_rect.min + self.camera.to_camera(path.points[i + 1].get_vec2());

                painter.line_segment([start_pos, end_pos], egui::Stroke::new(1.0, PATH_COLOR));

                if !matches!(self.path_edit_mode, EditMode::Edit) {
                    continue;
                }

                let color = if path.is_selected {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::WHITE
                };

                painter.circle_filled(start_pos, CIRCLE_RADIUS * self.camera.zoom, color);
                painter.circle_filled(end_pos, CIRCLE_RADIUS * self.camera.zoom, color);

                let start_rect = egui::Rect::from_center_size(
                    egui::Pos2::new(start_pos.x, start_pos.y - CIRCLE_RADIUS * 2.0),
                    egui::Vec2::splat(CIRCLE_RADIUS * self.camera.zoom),
                );

                let end_rect = egui::Rect::from_center_size(
                    egui::Pos2::new(end_pos.x, end_pos.y - CIRCLE_RADIUS * 2.0),
                    egui::Vec2::splat(CIRCLE_RADIUS * self.camera.zoom),
                );

                let start_resp = ui.interact(
                    canvas_rect.intersect(start_rect),
                    egui::Id::new(format!("path-{}-{}-start-{}", index, i, i % 2,)),
                    egui::Sense::click_and_drag(),
                );

                let end_resp = ui.interact(
                    canvas_rect.intersect(end_rect),
                    egui::Id::new(format!("path-{}-{}-end-{}", index, i, i % 2)),
                    egui::Sense::click_and_drag(),
                );

                let mut clicked = false;
                if start_resp.clicked() {
                    clicked = true;
                } else if start_resp.dragged() {
                    let world_delta = start_resp.drag_delta() / self.camera.zoom;
                    let start = &mut path.points[i];
                    start.x += world_delta.x;
                    start.y -= world_delta.y;
                }

                if end_resp.clicked() {
                    clicked = true;
                } else if end_resp.dragged() {
                    let world_delta = end_resp.drag_delta() / self.camera.zoom;
                    let end = &mut path.points[i + 1];
                    end.x += world_delta.x;
                    end.y -= world_delta.y;
                }

                if clicked {
                    self.selected_object_indices.push(ObjectIndex::Path(index));
                }
            }
        }
    }

    pub fn update_zones(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let painter = ui.painter_at(canvas_rect);

        for (index, zone) in self.current_mapdata.zones.iter_mut().enumerate() {
            if &zone.name == "NONE" && !self.display_none {
                continue;
            }

            let start = canvas_rect.min + self.camera.to_camera(zone.bounds_start.get_vec2());

            let end = canvas_rect.min + self.camera.to_camera(zone.bounds_end.get_vec2());

            let square = egui::Rect::from_points(&[start, end]);

            let body_resp = ui.interact(
                canvas_rect.intersect(square),
                egui::Id::new(zone as *const _),
                egui::Sense::click_and_drag(),
            );

            if zone.is_selected {
                painter.rect_filled(
                    square,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x10),
                );
            }

            painter.rect_stroke(
                Rect::from_points(&[start, end]),
                0.0,
                egui::Stroke::new(1.0, ZONE_COLOR),
            );

            if !matches!(self.zone_edit_mode, EditMode::Edit) {
                continue;
            }

            if body_resp.clicked() {
                self.selected_object_indices.push(ObjectIndex::Zone(index));
            } else if body_resp.dragged() {
                let world_delta = body_resp.drag_delta() / self.camera.zoom;

                zone.bounds_start.x += world_delta.x;
                zone.bounds_start.y -= world_delta.y;
                zone.bounds_end.x += world_delta.x;
                zone.bounds_end.y -= world_delta.y;
            }

            if !zone.is_selected {
                continue;
            }

            let start_rect = egui::Rect::from_center_size(
                egui::Pos2::new(start.x, start.y - CIRCLE_RADIUS * 2.0),
                egui::Vec2::splat(CIRCLE_RADIUS * self.camera.zoom),
            );

            let end_rect = egui::Rect::from_center_size(
                egui::Pos2::new(end.x, end.y - CIRCLE_RADIUS * 2.0),
                egui::Vec2::splat(CIRCLE_RADIUS * self.camera.zoom),
            );

            let color = egui::Color32::from_rgb(0x00, 0x9F, 0xFD);

            let radius = CIRCLE_RADIUS * 2.0;

            painter.circle_filled(start, radius * self.camera.zoom, color);
            painter.circle_filled(end, radius * self.camera.zoom, color);

            let start_resp = ui.interact(
                start_rect,
                egui::Id::new(&zone.bounds_start as *const _),
                egui::Sense::click_and_drag(),
            );

            let end_resp = ui.interact(
                end_rect,
                egui::Id::new(&zone.bounds_end as *const _),
                egui::Sense::click_and_drag(),
            );

            if start_resp.dragged() {
                let world_delta = start_resp.drag_delta() / self.camera.zoom;

                zone.bounds_start.x += world_delta.x;
                zone.bounds_start.y -= world_delta.y;
            }

            if end_resp.dragged() {
                let world_delta = end_resp.drag_delta() / self.camera.zoom;

                zone.bounds_end.x += world_delta.x;
                zone.bounds_end.y -= world_delta.y;
            }
        }
    }

    pub fn update_course_info(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let painter = ui.painter_at(canvas_rect);

        for (index, info) in self.current_mapdata.course_infos.iter_mut().enumerate() {
            if &info.name == "NONE" && !self.display_none {
                continue;
            }

            let pos = canvas_rect.min
                + self
                    .camera
                    .to_camera(info.position.get_point2d().get_vec2());

            let square = egui::Rect::from_center_size(
                egui::Pos2::new(pos.x, pos.y - SQUARE_SIZE * 2.0),
                egui::Vec2::splat(SQUARE_SIZE * self.camera.zoom),
            );

            let color = if info.is_selected {
                COURSE_INFO_COLOR
            } else {
                egui::Color32::WHITE
            };

            painter.rect_stroke(square, 0.0, egui::Stroke::new(1.0, color));

            if !matches!(self.course_info_edit_mode, EditMode::Edit) {
                continue;
            }

            let resp = ui.interact(
                canvas_rect.intersect(square),
                egui::Id::new(info as *const _),
                egui::Sense::click_and_drag(),
            );

            if resp.hovered() {
                painter.text(
                    pos + egui::Vec2::new(10.0, -10.0),
                    egui::Align2::LEFT_CENTER,
                    &info.name,
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );
            }

            if resp.clicked() {
                self.selected_object_indices
                    .push(ObjectIndex::CourseInfo(index));
            } else if resp.dragged() {
                let world_delta = resp.drag_delta() / self.camera.zoom;

                info.position.x += world_delta.x;
                info.position.y -= world_delta.y;
            }
        }
    }

    pub fn update_enemies(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let painter = ui.painter_at(canvas_rect);

        for (index, enemy) in self.current_endata.enemies.iter_mut().enumerate() {
            let pos = enemy.position_1.get_point2d();
            let screen_pos = canvas_rect.min.to_vec2() + self.camera.to_camera(pos.get_vec2());
            let square = egui::Rect::from_center_size(
                {
                    let pos = screen_pos.to_pos2();

                    egui::Pos2::new(pos.x, pos.y - SQUARE_SIZE * 2.0)
                },
                egui::Vec2::splat(SQUARE_SIZE * self.camera.zoom),
            );

            let resp = ui.interact(
                canvas_rect.intersect(square),
                egui::Id::new(enemy as *const _),
                egui::Sense::click_and_drag(),
            );

            let color = egui::Color32::from_rgb(0xE3, 0x96, 0xDF);

            painter.rect_stroke(square, 0.0, egui::Stroke::new(1.0, color));

            if resp.hovered() {
                painter.text(
                    screen_pos.to_pos2() + egui::Vec2::new(10.0, -10.0),
                    egui::Align2::LEFT_CENTER,
                    enemy_id_to_name(&enemy.name),
                    egui::FontId::default(),
                    egui::Color32::WHITE,
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
    pub fn process_wall_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
            self.current_mapdata.walls.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
            return;
        }

        let wall = &mut self.current_mapdata.walls[index];
        wall.is_selected = true;

        egui::Area::new(egui::Id::from("le_wall_attribute_editor"))
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui| {
                        ui.label("Edit wall attributes");

                        ui.label("Start Position");
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut wall.start.x)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut wall.start.y)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                        });

                        ui.label("End Position");
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut wall.end.x)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut wall.end.y)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                        });

                        egui::ComboBox::from_label("Collision Type")
                            .selected_text(&wall.collision_type)
                            .show_ui(ui, |ui| {
                                for collision_type in COLLISION_TYPES {
                                    ui.selectable_value(
                                        &mut wall.collision_type,
                                        collision_type.to_string(),
                                        collision_type,
                                    );
                                }
                            });
                        ui.add(
                            egui::TextEdit::singleline(&mut wall.collision_type).char_limit(0x20),
                        );
                    });
            });
    }

    pub fn process_labeled_wall_attributes(&mut self, ui: &egui::Ui, index: usize) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
            self.current_mapdata.walls.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
            return;
        }

        let wall = &mut self.current_mapdata.labeled_walls[index];
        wall.is_selected = true;

        egui::Area::new(egui::Id::from("le_labeled_wall_attribute_editor"))
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui| {
                        ui.label("Edit wall attributes");

                        ui.label("Start Position");
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut wall.start.x)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut wall.start.y)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                        });

                        ui.label("End Position");
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut wall.end.x)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut wall.end.y)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                        });

                        egui::ComboBox::from_label("Collision Type")
                            .selected_text(&wall.collision_type)
                            .show_ui(ui, |ui| {
                                for collision_type in COLLISION_TYPES {
                                    ui.selectable_value(
                                        &mut wall.collision_type,
                                        collision_type.to_string(),
                                        collision_type,
                                    );
                                }
                            });

                        ui.add(
                            egui::TextEdit::singleline(&mut wall.collision_type).char_limit(0x20),
                        );

                        ui.label("Label");
                        ui.add(egui::TextEdit::singleline(&mut wall.label).char_limit(0x20));
                    });
            });
    }

    fn process_copy_raw_bytes_context_menu(
        response: egui::Response,
        bytes: &[u8],
        is_string: bool,
    ) {
        // if the bytes are from a string and the string is empty then do nothing
        if bytes.is_empty() {
            return;
        }

        // if the bytes are from an int or a float but they're all zero then do nothing
        if !is_string && bytes.iter().all(|&byte| byte == 0) {
            return;
        }

        response.context_menu(|ui| {
            if is_string {
                // dealing with a string
                if ui.button("Copy raw bytes").clicked() {
                    let byte_string: String =
                        bytes.iter().map(|byte| format!("{:02X}", byte)).collect();

                    ui.ctx().copy_text(format!("0x{}", byte_string));
                    ui.close_menu();
                }
            } else {
                // allow for big- or little-endian copying
                let mut endian = None;

                if ui.button("Copy big-endian bytes").clicked() {
                    endian = Some(ByteOrder::Big);
                }

                if ui.button("Copy little-endian bytes").clicked() {
                    endian = Some(ByteOrder::Little);
                }

                if let Some(endian) = endian {
                    let byte_string = match endian {
                        ByteOrder::Big => bytes
                            .iter()
                            .rev()
                            .map(|byte| format!("{:02X}", byte))
                            .collect::<String>(),
                        ByteOrder::Little => bytes
                            .iter()
                            .map(|byte| format!("{:02X}", byte))
                            .collect::<String>(),
                    };

                    ui.ctx().copy_text(format!("0x{}", byte_string));
                    ui.close_menu();
                }
            }
        });
    }

    fn process_raw_int_param(ui: &mut egui::Ui, value: &mut i32) {
        let response = ui.add(
            egui::DragValue::new(value)
                .speed(1)
                .range(i32::MIN..=i32::MAX),
        );

        Self::process_copy_raw_bytes_context_menu(response, &value.to_le_bytes(), false);
    }

    fn process_raw_float_param(ui: &mut egui::Ui, value: &mut f32) {
        let response = ui.add(
            egui::DragValue::new(value)
                .speed(1)
                .range(f32::MIN..=f32::MAX),
        );

        Self::process_copy_raw_bytes_context_menu(response, &value.to_le_bytes(), false);
    }

    fn process_raw_string_param(ui: &mut egui::Ui, value: &mut String, char_limit: usize) {
        let response = ui.add(egui::TextEdit::singleline(value).char_limit(char_limit));

        Self::process_copy_raw_bytes_context_menu(response, value.as_bytes(), true);
    }

    pub fn process_common_gimmick_attributes(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
    ) -> Result<()> {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
            self.current_mapdata.common_gimmicks.remove(index);
            self.selected_object_indices.clear();
            return Ok(());
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
            return Ok(());
        }

        let gmk = &mut self.current_mapdata.common_gimmicks[index];

        gmk.is_selected = true;

        let (name, is_hex) = if let Some(n) = Self::get_translated_common_gimmick_name(
            self.object_data_json
                .as_ref()
                .context("object_data_json is None")?,
            &gmk.hex,
        ) {
            (n, false)
        } else {
            (gmk.hex.clone(), true)
        };

        egui::Area::new(egui::Id::from("le_common_gimmick_attribute_editor"))
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
            .inner_margin(egui::Vec2::splat(8.0))
            .show(ui, |ui|{
                ui.label("Edit common gimmick attributes");

                if is_hex {
                    ui.label(format!("Hex: {name}"));
                } else {
                    ui.label(format!("Name: {name}"));
                }

                ui.label("Position");
                ui.horizontal(|ui|{
                    ui.label("X");
                    ui.add(egui::DragValue::new(&mut gmk.position.x).speed(0.5).range(f32::MIN..=f32::MAX));
                    ui.label("Y");
                    ui.add(egui::DragValue::new(&mut gmk.position.y).speed(0.5).range(f32::MIN..=f32::MAX));
                    ui.label("Z");
                    ui.add(egui::DragValue::new(&mut gmk.position.z).speed(0.5).range(f32::MIN..=f32::MAX));
                });

                let data = self.object_data_json.as_ref().context("object_data_json is None")?.get("common_gimmicks")
                .expect("couldn't find 'common_gimmicks' in objectdata.json");

                if let Some(gmk_data) = data.get(&gmk.hex) {
                    if let Some(desc) = gmk_data.get("description").and_then(|d| d.as_str())
                        && !desc.is_empty() {
                            ui.label(desc);
                        }

                    if let Some(note) = gmk_data.get("note").and_then(|n| n.as_str())
                        && !note.is_empty() {
                            ui.label(format!("Note: {note}"));
                        }
                    
                    // regular parameters
                    if let Some(params) = gmk_data.get("parameters").and_then(|p| p.as_object()) {
                        for (param_name, param_data) in params {
                            ui.collapsing(param_name, |ui|{
                                if let Some(param_desc) = param_data.get("description").and_then(|d| d.as_str())
                                    && !param_desc.is_empty() {
                                        ui.label(param_desc);
                                    }

                                if let Some(param_note) = param_data.get("note").and_then(|n| n.as_str())
                                    && !param_note.is_empty() {
                                        ui.label(format!("Note: {param_note}"));
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

                                            if gmk.params.string_params[slot] == value.as_str().unwrap() {
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

                    // common parameters
                    if let Some(params) = gmk_data.get("common_parameters").and_then(|p| p.as_object()) {
                        for (param_name, param_data) in params {
                            ui.collapsing(param_name, |ui|{
                                if let Some(param_desc) = param_data.get("description").and_then(|d| d.as_str())
                                    && !param_desc.is_empty() {
                                        ui.label(param_desc);
                                    }

                                if let Some(param_note) = param_data.get("note").and_then(|n| n.as_str())
                                    && !param_note.is_empty() {
                                        ui.label(format!("Note: {param_note}"));
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
                                            egui::DragValue::new(&mut gmk.params.common_int_params[slot])
                                            .speed(1)
                                            .range(i32::MIN..=i32::MAX)
                                        );
                                    }

                                    DataType::Bool => {
                                        let mut bool_value = gmk.params.common_int_params[slot] != 0;
                                        if ui.checkbox(&mut bool_value, "Value").changed() {
                                            gmk.params.common_int_params[slot] = if bool_value { 1 } else { 0 }
                                        }
                                    }
                                    
                                    DataType::Float => {
                                        ui.add(
                                            egui::DragValue::new(&mut gmk.params.common_float_params[slot])
                                            .speed(1.0)
                                            .range(f32::MIN..=f32::MAX)
                                        );
                                    }

                                    DataType::String => {
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                &mut gmk.params.common_string_param
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

                                            if gmk.params.common_int_params[slot] == value.as_i64().unwrap() as i32 {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.common_int_params[slot],
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

                                            if gmk.params.common_float_params[slot] == value.as_f64().unwrap() as f32 {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.common_float_params[slot],
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

                                            if gmk.params.common_string_param == value.as_str().unwrap() {
                                                value_index = values.len() - 1;
                                            }
                                        }

                                        egui::ComboBox::from_label("Value")
                                        .selected_text(
                                            &values[value_index].0
                                        ).show_ui(ui, |ui|{
                                            for (value_name, value) in values {
                                                ui.selectable_value(
                                                    &mut gmk.params.common_string_param,
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
                            Self::process_raw_int_param(ui, &mut gmk.params.common_int_params[i]);
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("Float values (common)");
                    ui.horizontal(|ui|{
                        for i in 0..2 {
                            Self::process_raw_float_param(ui, &mut gmk.params.common_float_params[i]);
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("String value (common)");
                    Self::process_raw_string_param(ui, &mut gmk.params.common_string_param, 8);

                    ui.add_space(3.0);
                    ui.label("Int values");
                    ui.horizontal(|ui|{
                        for i in 0..5 {
                            Self::process_raw_int_param(ui, &mut gmk.params.int_params[i]);
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("Float values");
                    ui.horizontal(|ui|{
                        for i in 0..5 {
                            Self::process_raw_float_param(ui, &mut gmk.params.float_params[i]);
                        }
                    });

                    ui.add_space(3.0);
                    ui.label("String values");
                    for i in 0..5 {
                        Self::process_raw_string_param(ui, &mut gmk.params.string_params[i], 0x40);
                    }
                });

                Ok::<(), anyhow::Error>(())
            });
        });

        Ok(())
    }

    // common function for those with mapdata parameters
    fn process_mapdata_parameters(
        object_data_json: &serde_json::Value,
        ui: &mut egui::Ui,
        object_name: &str,
        object_category: &str,
        object_params: &mut Params,
    ) {
        let data = object_data_json
            .get(object_category)
            .unwrap_or_else(|| panic!("couldn't find '{object_category}' in objectdatajson"));

        // paramter handling
        if let Some(object_data) = data.get(object_name) {
            if let Some(desc) = object_data.get("description").and_then(|d| d.as_str())
                && !desc.is_empty()
            {
                ui.label(desc);
            }

            if let Some(note) = object_data.get("note").and_then(|n| n.as_str())
                && !note.is_empty()
            {
                ui.label(format!("Note: {note}"));
            }

            if let Some(params) = object_data.get("parameters").and_then(|p| p.as_object()) {
                for (param_name, param_data) in params {
                    ui.collapsing(param_name, |ui|{
                        if let Some(param_desc) = param_data.get("description").and_then(|d| d.as_str())
                            && !param_desc.is_empty() {
                                ui.label(param_desc);
                            }
                        if let Some(param_note) = param_data.get("note").and_then(|n| n.as_str())
                            && !param_note.is_empty() {
                                ui.label(format!("Note: {param_note}"));
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
                                    egui::DragValue::new(&mut object_params.int_params[slot])
                                    .speed(1)
                                    .range(i32::MIN..=i32::MAX)
                                );
                            }

                            DataType::Bool => {
                                let mut bool_value = object_params.int_params[slot] != 0;
                                if ui.checkbox(&mut bool_value, "Value").changed() {
                                    object_params.int_params[slot] = if bool_value { 1 } else { 0 }
                                }
                            }
                            
                            DataType::Float => {
                                ui.add(
                                    egui::DragValue::new(&mut object_params.float_params[slot])
                                    .speed(1.0)
                                    .range(f32::MIN..=f32::MAX)
                                );
                            }

                            DataType::String => {
                                ui.add(
                                    egui::TextEdit::singleline(
                                        &mut object_params.string_params[slot]
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

                                    if object_params.int_params[slot] == value.as_i64().unwrap() as i32 {
                                        value_index = values.len() - 1;
                                    }
                                }

                                egui::ComboBox::from_label("Value")
                                .selected_text(
                                    &values[value_index].0
                                ).show_ui(ui, |ui|{
                                    for (value_name, value) in values {
                                        ui.selectable_value(
                                            &mut object_params.int_params[slot],
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

                                    if object_params.float_params[slot] == value.as_f64().unwrap() as f32 {
                                        value_index = values.len() - 1;
                                    }
                                }

                                egui::ComboBox::from_label("Value")
                                .selected_text(
                                    &values[value_index].0
                                ).show_ui(ui, |ui|{
                                    for (value_name, value) in values {
                                        ui.selectable_value(
                                            &mut object_params.float_params[slot],
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

                                    if object_params.string_params[slot] == value.as_str().unwrap() {
                                        value_index = values.len() - 1;
                                    }
                                }

                                egui::ComboBox::from_label("Value")
                                .selected_text(
                                    &values[value_index].0
                                ).show_ui(ui, |ui|{
                                    for (value_name, value) in values {
                                        ui.selectable_value(
                                            &mut object_params.string_params[slot],
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

        ui.collapsing("Raw data", |ui| {
            ui.label("Edit fields regardless of documentation.");

            ui.add_space(3.0);
            ui.label("Int values");
            ui.horizontal(|ui| {
                for i in 0..3 {
                    Self::process_raw_int_param(ui, &mut object_params.int_params[i]);
                }
            });

            ui.add_space(3.0);
            ui.label("Float values");
            ui.horizontal(|ui| {
                for i in 0..3 {
                    Self::process_raw_float_param(ui, &mut object_params.float_params[i]);
                }
            });

            ui.add_space(3.0);
            ui.label("String values");
            for i in 0..3 {
                Self::process_raw_string_param(ui, &mut object_params.string_params[i], 0x40);
            }
        });
    }

    pub fn process_gimmick_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
            self.current_mapdata.gimmicks.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
            return;
        }

        let gmk = &mut self.current_mapdata.gimmicks[index];

        gmk.is_selected = true;

        egui::Area::new(egui::Id::from("le_gimmick_attribute_editor"))
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui| {
                        ui.label("Edit gimmick attributes");
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            ui.add(egui::TextEdit::singleline(&mut gmk.name).char_limit(0x30));
                        });

                        ui.label("Position");
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut gmk.position.x)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut gmk.position.y)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Z");
                            ui.add(
                                egui::DragValue::new(&mut gmk.position.z)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                        });

                        Self::process_mapdata_parameters(
                            self.object_data_json
                                .as_ref()
                                .context("object_data_json is None")?,
                            ui,
                            &gmk.name,
                            "gimmicks",
                            &mut gmk.params,
                        );

                        Ok::<(), anyhow::Error>(())
                    });
            });
    }

    pub fn process_path_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
            self.current_mapdata.paths.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
            return;
        }

        let path = &mut self.current_mapdata.paths[index];
        path.is_selected = true;

        egui::Area::new(egui::Id::from("le_path_attribute_editor"))
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui| {
                        ui.label("Edit path attributes");

                        ui.label("Name");
                        ui.add(egui::TextEdit::singleline(&mut path.name).char_limit(0x20));

                        let data = self
                            .object_data_json
                            .as_ref()
                            .context("object_data_json is None")?
                            .get("paths")
                            .expect("couldn't find 'paths' in objectdata.json");

                        if let Some(path_data) = data.get(&path.name) {
                            if let Some(desc) =
                                path_data.get("description").and_then(|d| d.as_str())
                                && !desc.is_empty()
                            {
                                ui.label(desc);
                            }

                            if let Some(note) = path_data.get("note").and_then(|n| n.as_str())
                                && !note.is_empty()
                            {
                                ui.label(format!("Note: {note}"));
                            }
                        }

                        ui.collapsing("Points", |ui| {
                            let max_height = 200.0;
                            let max_width = ui.available_width();

                            let mut to_remove = None;
                            let mut to_insert = None;
                            let num_points = path.points.len();

                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(max_width, max_height),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    let scroll_area =
                                        egui::ScrollArea::vertical().max_height(max_height);

                                    scroll_area.show(ui, |ui| {
                                        for (index, point) in path.points.iter_mut().enumerate() {
                                            ui.horizontal(|ui| {
                                                if ui.button("+").clicked() {
                                                    to_insert = Some(index);
                                                }

                                                if num_points > 2
                                                    && index != 0
                                                    && ui.button("-").clicked()
                                                {
                                                    to_remove = Some(index);
                                                }

                                                ui.label(format!("{}", index + 1));
                                                ui.add_space(3.0);
                                                ui.label("X");
                                                ui.add(
                                                    egui::DragValue::new(&mut point.x)
                                                        .speed(0.5)
                                                        .range(f32::MIN..=f32::MAX),
                                                );
                                                ui.label("Y");
                                                ui.add(
                                                    egui::DragValue::new(&mut point.y)
                                                        .speed(0.5)
                                                        .range(f32::MIN..=f32::MAX),
                                                );
                                            });
                                        }
                                    });

                                    if let Some(remove_index) = to_remove {
                                        path.points.remove(remove_index);
                                    }

                                    if let Some(insert_index) = to_insert {
                                        path.points
                                            .insert(insert_index + 1, path.points[insert_index]);
                                    }
                                },
                            );
                        });

                        Self::process_mapdata_parameters(
                            self.object_data_json
                                .as_ref()
                                .context("object_data_json is None")?,
                            ui,
                            &path.name,
                            "paths",
                            &mut path.params,
                        );

                        Ok::<(), anyhow::Error>(())
                    });
            });
    }

    pub fn process_zone_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
            self.current_mapdata.zones.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
            return;
        }

        let zone = &mut self.current_mapdata.zones[index];

        zone.is_selected = true;

        egui::Area::new(egui::Id::from("le_zone_attribute_editor"))
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui| {
                        ui.label("Edit zone attributes");

                        ui.label("Name");
                        ui.add(egui::TextEdit::singleline(&mut zone.name).char_limit(0x20));

                        let data = self
                            .object_data_json
                            .as_ref()
                            .context("object_data_json is None")?
                            .get("zones")
                            .expect("couldn't find 'zones' in objectdata.json");

                        if let Some(zone_data) = data.get(&zone.name) {
                            if let Some(desc) =
                                zone_data.get("description").and_then(|d| d.as_str())
                                && !desc.is_empty()
                            {
                                ui.label(desc);
                            }

                            if let Some(note) = zone_data.get("note").and_then(|n| n.as_str())
                                && !note.is_empty()
                            {
                                ui.label(format!("Note: {note}"));
                            }
                        }

                        ui.label("Unknown @ 0x20");
                        ui.add(egui::TextEdit::singleline(&mut zone.unk_20).char_limit(0x20));

                        ui.label("Bounds Start");
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut zone.bounds_start.x)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );

                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut zone.bounds_start.y)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                        });

                        ui.label("Bounds End");
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut zone.bounds_end.x)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );

                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut zone.bounds_end.y)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX),
                            );
                        });

                        Self::process_mapdata_parameters(
                            self.object_data_json
                                .as_ref()
                                .context("object_data_json is None")?,
                            ui,
                            &zone.name,
                            "zones",
                            &mut zone.params,
                        );

                        Ok::<(), anyhow::Error>(())
                    });
            });
    }

    pub fn process_course_info_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
            self.current_mapdata.course_infos.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
            return;
        }

        let info = &mut self.current_mapdata.course_infos[index];
        info.is_selected = true;

        egui::Area::new(egui::Id::from("le_course_info_attribute_editor"))
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui| {
                        ui.label("Edit course info attributes");

                        ui.label("Name");
                        ui.add(egui::TextEdit::singleline(&mut info.name).char_limit(0x20));

                        let data = self
                            .object_data_json
                            .as_ref()
                            .context("object_data_json is None")?
                            .get("course_infos")
                            .expect("couldn't find 'course_infos' in objectdata.json");

                        if let Some(course_info_data) = data.get(&info.name) {
                            if let Some(desc) =
                                course_info_data.get("description").and_then(|d| d.as_str())
                                && !desc.is_empty()
                            {
                                ui.label(desc);
                            }

                            if let Some(note) =
                                course_info_data.get("note").and_then(|n| n.as_str())
                                && !note.is_empty()
                            {
                                ui.label(format!("Note: {note}"));
                            }
                        }

                        ui.label("Unknown @ 0x20");
                        ui.add(egui::TextEdit::singleline(&mut info.unk_20).char_limit(0x20));

                        Self::process_mapdata_parameters(
                            self.object_data_json
                                .as_ref()
                                .context("object_data_json is None")?,
                            ui,
                            &info.name,
                            "course_infos",
                            &mut info.params,
                        );

                        Ok::<(), anyhow::Error>(())
                    });
            });
    }

    pub fn process_enemy_attributes(&mut self, ui: &mut egui::Ui, index: usize) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
            self.current_endata.enemies.remove(index);
            self.selected_object_indices.clear();
            return;
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.deselect_all();
            return;
        }

        let enemy = &mut self.current_endata.enemies[index];

        enemy.is_selected = true;

        egui::Area::new(egui::Id::from("le_enemy_attribute_editor"))
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Vec2::splat(8.0))
                    .show(ui, |ui| {
                        ui.label("Edit enemy attributes");

                        egui::ComboBox::from_label("Enemy")
                            .selected_text(enemy_id_to_name(&enemy.name))
                            .show_ui(ui, |ui| {
                                for (id, name) in ENEMY_LIST {
                                    ui.selectable_value(&mut enemy.name, String::from(id), name);
                                }
                            });

                        egui::ComboBox::from_label("Behavior")
                            .selected_text(&enemy.behavior)
                            .show_ui(ui, |ui| {
                                let behaviors = [
                                    "STAND",
                                    "WALK1",
                                    "WALK2",
                                    "WALK_AREA",
                                    "JUMP",
                                    "JUMP_LR",
                                    "FLOAT",
                                    "UP_DOWN",
                                    "SLIDE",
                                    "SEARCH",
                                    "ATTACK1",
                                    "ATTACK2",
                                    "ATTACK3",
                                    "READER",
                                    "FOLLOWING",
                                    "PURSUE",
                                    "ESCAPE",
                                    "DEMO",
                                    "EVENT",
                                ];

                                for behavior in behaviors {
                                    ui.selectable_value(
                                        &mut enemy.behavior,
                                        String::from(behavior),
                                        behavior,
                                    );
                                }
                            });

                        ui.horizontal(|ui| {
                            ui.label("Path name");
                            ui.add(
                                egui::TextEdit::singleline(&mut enemy.path_name).char_limit(0x20),
                            );
                        });

                        egui::ComboBox::from_label("Bead type")
                            .selected_text(&enemy.bead_type)
                            .show_ui(ui, |ui| {
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
                            .selected_text(color_string_to_label(&enemy.bead_color))
                            .show_ui(ui, |ui| {
                                for color in [
                                    "Red", "Orange", "Yellow", "Green", "Blue", "Purple", "White",
                                    "Random",
                                ] {
                                    ui.selectable_value(
                                        &mut enemy.bead_color,
                                        label_to_color_string(color),
                                        color,
                                    );
                                }
                            });

                        egui::ComboBox::from_label("Direction")
                            .selected_text(&enemy.direction)
                            .show_ui(ui, |ui| {
                                let dirs = ["RIGHT", "LEFT", "UP", "DOWN"];

                                for dir in dirs {
                                    ui.selectable_value(
                                        &mut enemy.direction,
                                        String::from(dir),
                                        dir,
                                    );
                                }
                            });

                        ui.horizontal(|ui| {
                            ui.label("Unknown @ 0x88");
                            ui.add(egui::TextEdit::singleline(&mut enemy.unk_88).char_limit(8));
                        });

                        egui::ComboBox::from_label("Orientation")
                            .selected_text(&enemy.orientation)
                            .show_ui(ui, |ui| {
                                let orientations = ["NONE", "FRONT", "BACK"];

                                for orientation in orientations {
                                    ui.selectable_value(
                                        &mut enemy.orientation,
                                        String::from(orientation),
                                        orientation,
                                    );
                                }
                            });

                        ui.collapsing("Parameters", |ui| {
                            for i in 0..7 {
                                ui.collapsing(format!("Set {}", i + 1), |ui| {
                                    ui.label("Float values");
                                    ui.horizontal(|ui| {
                                        for j in 0..3 {
                                            ui.add(
                                                egui::DragValue::new(
                                                    &mut enemy.params[i].float_params[j],
                                                )
                                                .speed(1.0)
                                                .range(f32::MIN..=f32::MAX),
                                            );
                                        }
                                    });

                                    ui.label("Int values");
                                    ui.horizontal(|ui| {
                                        for j in 0..3 {
                                            ui.add(
                                                egui::DragValue::new(
                                                    &mut enemy.params[i].int_params[j],
                                                )
                                                .speed(1.0)
                                                .range(f32::MIN..=f32::MAX),
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
