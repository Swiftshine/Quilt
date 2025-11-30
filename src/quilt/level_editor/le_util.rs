use super::{
    EditMode, LevelEditor, ObjectIndex
};

use std::{env, fs};
use anyhow::{bail, Result, Context};
use reqwest::blocking::Client;

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
        if let std::collections::hash_map::Entry::Vacant(e) = self.object_textures.entry(key.clone()) {
            if let Ok(image_data) = Self::load_image_from_tex_folder("common_gimmick", hex) {
                let texture = ctx.load_texture(
                    &key, image_data, egui::TextureOptions::LINEAR
                );
                e.insert(texture);
            }
        }
    }

    pub fn load_image_from_tex_folder(folder_name: &str, file_name: &str) -> Result<egui::ColorImage> {
        let path = format!("quilt_res/tex/{folder_name}/{file_name}.png");
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
            if let std::collections::hash_map::Entry::Vacant(e) = self.object_textures.entry(key.clone()) {
                if let Ok(image_data) = Self::load_image_from_tex_folder("gimmick", &gmk.name) {
                    let texture = ctx.load_texture(
                        &key, image_data, egui::TextureOptions::LINEAR
                    );

                    e.insert(texture);
                }
            }
        }

        for gmk in self.current_mapdata.common_gimmicks.iter() {
            let key = format!("common_gimmick-{}", &gmk.hex);
            if let std::collections::hash_map::Entry::Vacant(e) = self.object_textures.entry(key.clone()) {
                if let Ok(image_data) = Self::load_image_from_tex_folder("common_gimmick", &gmk.hex) {
                    let texture = ctx.load_texture(
                        &key, image_data, egui::TextureOptions::LINEAR
                    );
                    e.insert(texture);
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

                ObjectIndex::LabeledWall(index) => {
                    self.current_mapdata.labeled_walls[*index].is_selected = false;
                }       
                         
                ObjectIndex::CommonGimmick(index) => {
                    self.current_mapdata.common_gimmicks[*index].is_selected = false;
                }

                ObjectIndex::Gimmick(index) => {
                    self.current_mapdata.gimmicks[*index].is_selected = false;
                }

                ObjectIndex::Path(index) => {
                    self.current_mapdata.paths[*index].is_selected = false;
                }
                
                ObjectIndex::Zone(index) => {
                    self.current_mapdata.zones[*index].is_selected = false;
                }

                ObjectIndex::CourseInfo(index) => {
                    self.current_mapdata.course_infos[*index].is_selected = false;
                }

                ObjectIndex::Enemy(index) => {
                    self.current_endata.enemies[*index].is_selected = false;
                }
            }
        }
        self.selected_object_indices.clear();
    }

    pub fn update_object_data(&mut self) -> Result<(), anyhow::Error> {
        let client = Client::new();

        let response = client
        .get("https://raw.githubusercontent.com/Swiftshine/key-objectdb/refs/heads/main/objectdata.json")
        .send()?;

        if !response.status().is_success() {
            bail!("failed to update objectdata.json");
        }

        let content = response.text()?;

        let current_exe = env::current_exe()?;
        let current_dir = current_exe.parent().context("failed to get parent directory")?;

        let quilt_res_path = current_dir.join("quilt_res");

        if let Ok(b) = fs::exists(&quilt_res_path) {
            if !b {
                fs::create_dir(&quilt_res_path)?;
            }
            
            fs::write(quilt_res_path.join("objectdata.json"), &content)?;
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

    pub fn refresh_object_data(&mut self) -> Result<()> {
        let current_exe = env::current_exe()?;
        let current_dir = current_exe.parent().context("failed to get parent directory")?;
        let file_path = current_dir.join("quilt_res").join("objectdata.json");
        if let Ok(s) = fs::read_to_string(file_path) {
            self.object_data_json = serde_json::from_str(&s).expect("failed to read json");
        }

        Ok(())
    }
}
