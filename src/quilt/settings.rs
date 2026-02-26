use anyhow::Result;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{env, fmt::Display, fs};

#[derive(Serialize, Deserialize, PartialEq)]
struct QuiltVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl QuiltVersion {
    /// This version is to match the latest version of Quilt in which the available settings
    /// were updated.
    const fn latest() -> Self {
        // 1.2.3
        Self {
            major: 1,
            minor: 2,
            patch: 3,
        }
    }
}

impl Display for QuiltVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LevelEditorSettings {
    /// When a level is loaded, the camera will snap to the `START` gimmick, if present.
    pub snap_to_start: bool,
}

impl Default for LevelEditorSettings {
    fn default() -> Self {
        Self {
            snap_to_start: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct QuiltSettings {
    version: QuiltVersion,
    pub level_editor_settings: LevelEditorSettings,
}

impl Default for QuiltSettings {
    fn default() -> Self {
        Self {
            version: QuiltVersion::latest(),
            level_editor_settings: LevelEditorSettings::default(),
        }
    }
}

impl QuiltSettings {
    pub fn load_settings() -> Result<Self> {
        // todo: check settings and warn the user that stuff's gonna be updated

        let current_exe = env::current_exe().expect("failed to get current executable path");
        let current_dir = current_exe
            .parent()
            .expect("failed to get parent directory");
        let file_path = current_dir.join("quilt_res").join("quilt_settings.json");

        // check if exists
        if fs::exists(&file_path)? {
            // actually load the data

            let json_contents = fs::read_to_string(file_path)?;
            let mut settings = serde_json::from_str::<QuiltSettings>(&json_contents)?;

            // serde_json ignores fields that aren't part of the original struct
            // which allows us to handle versions

            if settings.version != QuiltVersion::latest() {
                // todo: warn the user that the version will update
                // should they choose to save the file

                // for now, we can just print it
                println!(
                    "Warning: When opening quilt_settings.json, the version {} was found. Upon saving, quilt_settings.json will be adjusted to fit the current version ({}).",
                    settings.version,
                    QuiltVersion::latest()
                );

                settings.version = QuiltVersion::latest();
            }

            Ok(settings)
        } else {
            // brand new settings
            Ok(Self::default())
        }
    }

    pub fn save_settings(&self) -> Result<()> {
        let current_exe = env::current_exe().expect("failed to get current executable path");
        let current_dir = current_exe
            .parent()
            .expect("failed to get parent directory");
        let file_path = current_dir.join("quilt_res").join("quilt_settings.json");

        // check folder
        if !fs::exists(current_dir.join("quilt_res"))? {
            fs::create_dir(current_dir.join("quilt_res"))?;
        }

        let contents = serde_json::to_string_pretty(&self)?;

        fs::write(file_path, contents)?;

        Ok(())
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        if ui.button("Save").clicked() {
            let _ = self.save_settings();
        }

        // level editor settings
        ui.label("Level Editor");
        ui.separator();

        ui.checkbox(
            &mut self.level_editor_settings.snap_to_start,
            "Snap to \"START\"?",
        )
        .on_hover_text(
            "When a level is loaded, the camera will snap to the \"START\" gimmick, if present.",
        );
    }
}
