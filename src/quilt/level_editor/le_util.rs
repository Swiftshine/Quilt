use super::{
    LevelEditor,
    ObjectIndex
};

use std::fs;
use anyhow::{bail, Result};
use reqwest::blocking::Client;
use serde_json;

impl LevelEditor {
    pub fn deselect_all(&mut self) {
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
