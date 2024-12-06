use super::{
    EditMode, LevelEditor, ObjectIndex
};

use std::fs;
use anyhow::{bail, Result};
use reqwest::blocking::Client;
use serde_json;

impl LevelEditor {
    pub fn edit_mode_to_string(edit_mode: EditMode) -> String {
        match edit_mode {
            EditMode::Hide => "Hide",
            EditMode::View => "View",
            EditMode::Edit => "Edit"
        }.to_string()
    }
    
    pub fn add_common_gimmick_texture(&mut self, ctx: &egui::Context, hex: &str) {
        let key = format!("common_gimmick-{}", hex);
        if !self.object_textures.contains_key(&key) {
                if let Ok(image_data) = Self::load_image_from_tex_folder("common_gimmick", hex) {
                let texture = ctx.load_texture(
                    &key, image_data, egui::TextureOptions::LINEAR
                );
                self.object_textures.insert(key, texture);
            }
        }
    }

    pub fn load_image_from_tex_folder(folder_name: &str, file_name: &str) -> Result<egui::ColorImage> {
        let path = format!("res/tex/{folder_name}/{file_name}.png");
        let image = image::open(&path)?.to_rgba8();

        let (width, height) = image.dimensions();
        let pixels = image
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        Ok(egui::ColorImage {
                size: [width as usize, height as usize],
                pixels
            }
        )
    }

    pub fn load_object_textures(&mut self, ctx: &egui::Context) {
        // gimmicks

        for gmk in self.current_mapdata.gimmicks.iter() {
            let key = format!("gimmick-{}", &gmk.name);
            if !self.object_textures.contains_key(&key) {
                if let Ok(image_data) = Self::load_image_from_tex_folder("gimmick", &gmk.name) {
                    let texture = ctx.load_texture(
                        &key, image_data, egui::TextureOptions::LINEAR
                    );

                    self.object_textures.insert(key, texture);
                }
            }
        }

        for gmk in self.current_mapdata.common_gimmicks.iter() {
            let key = format!("common_gimmick-{}", &gmk.hex);
            if !self.object_textures.contains_key(&key) {
                if let Ok(image_data) = Self::load_image_from_tex_folder("common_gimmick", &gmk.hex) {
                    let texture = ctx.load_texture(
                        &key, image_data, egui::TextureOptions::LINEAR
                    );

                    self.object_textures.insert(key, texture);
                }
            }
        }
    }

    pub fn deselect_all(&mut self) {
        for select_type in self.selected_object_indices.iter() {
            match select_type {
                ObjectIndex::Wall(index) => {
                    self.current_mapdata.walls[*index].is_selected = false;
                }
                
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

    pub fn update_object_data(&mut self) -> Result<()> {
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

    pub fn get_translated_common_gimmick_name(json: &serde_json::Value, hex: &str) -> Option<String> {
        let common = json.get("common_gimmicks")?;
        let data = common.get(hex)?;
        let translation = data.get("name").and_then(|s| s.as_str())?;
        Some(translation.to_string())
    }

    pub fn refresh_object_data(&mut self) {
        if let Ok(s) = fs::read_to_string("res/objectdata.json") {
            self.object_data_json = serde_json::from_str(&s).expect("failed to read json");
        }
    }
}
