use super::LevelEditor;
use crate::quilt::game::{endata::*, mapdata::*};
use anyhow::{Result, bail};
use gfarch::gfarch;
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

impl LevelEditor {
    pub fn update_level_data(&mut self) {
        self.current_endata = if let Some(enbin_index) = self.selected_enbin_index {
            Endata::from_data(&self.archive_contents[enbin_index].1)
        } else {
            Default::default()
        };

        self.current_mapdata = if let Some(mapbin_index) = self.selected_mapbin_index {
            Mapdata::from_data(&self.archive_contents[mapbin_index].1)
        } else {
            Default::default()
        };

        self.selected_object_indices.clear();
    }

    pub fn open_file(&mut self, ctx: &egui::Context) -> Result<()> {
        if let Some(path) = FileDialog::new()
            .add_filter("Level archive", &["gfa"])
            .pick_file()
        {
            self.file_path = Some(path);
            let path = self.file_path.as_ref().unwrap();
            let data = fs::read(path)?;
            self.archive_contents = gfarch::extract(&data)?;

            if self.archive_contents.is_empty() {
                bail!("archive is invalid");
            }

            self.selected_enbin_index = if self.archive_contents[0].0.contains(".enbin") {
                Some(0)
            } else {
                None
            };

            self.selected_mapbin_index = if self.archive_contents.len() > 1
                && self.archive_contents[1].0.contains(".mapbin")
            {
                Some(1)
            } else {
                None
            };

            self.selected_file_index = 0;

            self.update_level_data();

            self.file_open = true;

            // images
            self.object_textures.clear();

            self.load_object_textures(ctx);

            self.bgst_renderer.bgst_file = None;
            self.render_bgst = false;
        }

        Ok(())
    }

    pub fn open_folder(&mut self, ctx: &egui::Context) -> Result<()> {
        if let Some(path) = FileDialog::new().pick_folder() {
            self.file_path = Some(path.clone());

            if let Ok(entries) = fs::read_dir(&path) {
                let mut files = Vec::new();

                for entry in entries.flatten() {
                    let filepath = entry.path();
                    let filepath_str = filepath.to_str().unwrap();
                    if filepath_str.contains(".enbin") || filepath_str.contains(".mapbin") {
                        let contents = fs::read(&filepath).unwrap();

                        let filename = filepath.file_name().unwrap().to_str().unwrap().to_string();

                        files.push((filename, contents));
                    }
                }

                if files.is_empty() {
                    bail!("no files found in the folder");
                }

                self.archive_contents = files;

                self.selected_enbin_index = if self.archive_contents[0].0.contains(".enbin") {
                    Some(0)
                } else {
                    None
                };

                self.selected_mapbin_index = if self.archive_contents.len() > 1
                    && self.archive_contents[1].0.contains(".mapbin")
                {
                    Some(1)
                } else {
                    None
                };

                self.selected_file_index = 0;
                self.update_level_data();
                self.file_open = true;
                self.object_textures.clear();
                self.load_object_textures(ctx);
                self.bgst_renderer.bgst_file = None;
                self.render_bgst = false;
            }
        }
        Ok(())
    }

    pub fn save_file(&mut self, save_as: bool) -> Result<()> {
        if save_as {
            match rfd::FileDialog::new()
                .add_filter("Level archive", &["gfa"])
                .save_file()
            {
                Some(p) => {
                    self.file_path = Some(p);
                }

                None => {
                    bail!("User exited.");
                }
            }
        }

        // enbin
        if let Some(index) = self.selected_enbin_index {
            self.archive_contents[index].1 = self.current_endata.to_bytes();
        }

        // mapbin
        if let Some(index) = self.selected_mapbin_index {
            self.archive_contents[index].1 = self.current_mapdata.get_bytes();
        }

        let archive = gfarch::pack_from_files(
            &self.archive_contents,
            gfarch::Version::V3,
            gfarch::CompressionType::BPE,
            gfarch::GFCPOffset::Default,
        );

        fs::write(self.file_path.as_ref().unwrap(), archive)?;
        Ok(())
    }

    pub fn save_folder(&mut self, save_as: bool) -> Result<()> {
        if save_as {
            match rfd::FileDialog::new().pick_folder() {
                Some(p) => {
                    self.file_path = Some(p);
                }

                None => {
                    bail!("User exited.");
                }
            }
        }

        let stem = match self.file_path.as_ref().unwrap().file_stem() {
            Some(p) => PathBuf::from(p.to_str().unwrap_or_default()),

            None => {
                bail!("Couldn't get file stem");
            }
        };

        // enbin
        if let Some(index) = self.selected_enbin_index {
            self.archive_contents[index].1 = self.current_endata.to_bytes();
        }

        // mapbin
        if let Some(index) = self.selected_mapbin_index {
            self.archive_contents[index].1 = self.current_mapdata.get_bytes();
        }

        for file in self.archive_contents.iter() {
            let filepath = self
                .file_path
                .as_ref()
                .unwrap()
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/"
                + stem.to_str().unwrap()
                + "/"
                + &file.0;

            let _ = fs::write(filepath, &file.1);
        }

        Ok(())
    }

    pub fn make_new(&mut self) {
        self.file_open = true;
        self.file_path = None;
        self.archive_contents.clear();
        self.archive_contents
            .push((String::from("1.enbin"), Endata::default().to_bytes()));
        self.archive_contents
            .push((String::from("1.mapbin"), Mapdata::default().get_bytes()));
        self.update_level_data();
    }
}
