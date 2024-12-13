use super::LevelEditor;
use std::fs;
use std::path::PathBuf;
use anyhow::{bail, Result};
use ::gfarch::gfarch::FileContents;
use rfd::FileDialog;
use gfarch::gfarch;
use crate::quilt::game::{
    endata::*,
    mapdata::*
};

impl LevelEditor {
    
    pub fn update_level_data(&mut self) {
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

    pub fn open_file(&mut self, ctx: &egui::Context) -> Result<()> {
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
            self.file_path = path.clone();
            
            if let Ok(entries) = fs::read_dir(&path) {
                let mut files = Vec::new();

                for entry in entries {
                    if let Ok(entry) = entry {
                        let filepath = entry.path();
                        let filepath_str = filepath.to_str().unwrap();
                        if filepath_str.contains(".enbin") || filepath_str.contains(".mapbin") {
                            let contents = fs::read(&filepath).unwrap();
                            
                            let filename = filepath.file_name().unwrap().to_str().unwrap().to_string();

                            let file = FileContents {
                                contents,
                                filename
                            };

                            files.push(file);
                        }
                    }
                }

                if files.len() == 0 {
                    bail!("no files found in the folder");
                }

                self.archive_contents = files;
                
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
            .save_file() {
                Some(p) => {
                    self.file_path = p;
                }

                None => {
                    bail!("User exited.");
                }
            }
        }

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

    pub fn save_folder(&mut self, save_as: bool) -> Result<()> {
        if save_as {
            match rfd::FileDialog::new()
            .pick_folder() {
                Some(p) => {
                    self.file_path = p;
                }

                None => {
                    bail!("User exited.");
                }
            }
        }

        let stem = match self.file_path.file_stem() {
            Some(p) => {
                PathBuf::from(p.to_str().unwrap_or_default())
            }
            
            None => {
                bail!("Couldn't get file stem");
            }
        };


        // enbin
        if let Some(index) = self.selected_enbin_index {
            self.archive_contents[index].contents = self.current_endata.to_bytes();
        }

        // mapbin
        if let Some(index) = self.selected_mapbin_index {
            self.archive_contents[index].contents = self.current_mapdata.to_bytes();
        }
        for file in self.archive_contents.iter() {
            let filepath =
            self.file_path.parent().unwrap().to_str().unwrap().to_string() +
            "/" +
            stem.to_str().unwrap() +
            "/" +
            &file.filename;

            let _ = fs::write(filepath, &file.contents);
        }

        Ok(())
    }
}
