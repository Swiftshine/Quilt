use super::LevelEditor;
use std::fs;
use anyhow::{bail, Result};
use rfd::FileDialog;
use gfarch::gfarch;
use super::endata::Endata;
use super::mapdata::Mapdata;

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
        }

        Ok(())
    }

    pub fn save_file(&mut self, save_as: bool) -> Result<()> {
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

}
