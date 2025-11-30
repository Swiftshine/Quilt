use anyhow::Result;
use rfd::FileDialog;
use std::fs;
use gfarch::gfarch;

#[derive(Default)]
pub struct GfArchUtility {

}

impl GfArchUtility {
    pub fn new() -> Self {
        Self { }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("gu_top_panel")
        .show(ui.ctx(), |ui|{
            egui::menu::bar(ui, |ui|{

                if ui.button("Extract Archive").clicked() {
                    let _ = self.extract_archive();
                    ui.close_menu();
                }

                if ui.button("Create Archive (Wii)").clicked() {
                    let _ = self.create_archive(gfarch::CompressionType::BPE, gfarch::Version::V3);
                    ui.close_menu();
                }
            });
        });
    }

    fn extract_archive(&self) -> Result<()> {
        // ask the user to open an archive to extract
        if let Some(archive_path) = FileDialog::new()
        .add_filter("Good-Feel Archive", &["gfa"])
        .pick_file() { 
            // ask user to pick a folder to extract to
            if let Some(output_folder_path) = FileDialog::new().pick_folder() {
                let raw_archive = fs::read(archive_path)?;
                let archive_contents = gfarch::extract(&raw_archive)?;

                let mut num_failed = 0;

                let output_folder_path = output_folder_path.to_str().unwrap().to_string();
                
                println!("{}", output_folder_path);

                for file in archive_contents {
                    if fs::write(output_folder_path.clone() + "/" + &file.0, &file.1).is_err() {
                        num_failed += 1;
                    }
                }

                if num_failed != 0 {
                    eprintln!("Failed to extract {} files when reading archive", num_failed);
                }
            }
        }

        Ok(())
    }

    fn create_archive(&self, compression_type: gfarch::CompressionType, version: gfarch::Version) -> Result<()> {
        // ask user to open a folder to collect files from
        if let Some(input_folder_path) = FileDialog::new().pick_folder() {
            // ask user to pick an archive name
            if let Some(archive_name) = FileDialog::new().add_filter("Good-Feel Archive", &["gfa"]).save_file() {
                // read contents of the folder

                let mut num_failed = 0;

                let mut files: Vec<(String, Vec<u8>)> = Vec::new();

                for entry in fs::read_dir(input_folder_path)? {
                    let entry = if let Ok(entry) = entry{
                        entry
                    } else {
                        num_failed += 1;
                        continue;
                    };

                    let path = entry.path();
                    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();

                    // strip the filena
                    let contents = fs::read(path)?;

                    files.push((filename, contents));
                }

                if files.is_empty() {
                    // nothing to do
                    return Ok(());
                }
                
                let archive = gfarch::pack_from_files(&files, version, compression_type, gfarch::GFCPOffset::Default);

                if num_failed != 0 {
                    eprintln!("Failed to read {} files when creating archive", num_failed);
                }

                fs::write(archive_name, archive)?;
            }
        }

        Ok(())
    }
}