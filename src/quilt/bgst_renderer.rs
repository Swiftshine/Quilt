mod bgst;
use bgst::BGSTFile;
use anyhow::{bail, Result};
use rfd::FileDialog;
use std::fs;

#[derive(Default)]
pub struct BGSTRenderer {
    pub bgst_file: Option<BGSTFile>,
    pub decoded_image_handles: Vec<egui::TextureHandle>,
    pub tile_size: f32,
    pub tile_offset: egui::Vec2,
    pub tile_scale: egui::Vec2,

}


impl BGSTRenderer {
    pub fn new() -> Self {
        Self {
            tile_size: 11.9,
            tile_scale: egui::Vec2::new(1.028, 1.019),
            tile_offset: egui::Vec2::splat(0.0),

            ..Default::default()
        }
    }

    pub fn open_file(&mut self, ui: &egui::Ui) -> Result<()> {
        if let Some(path) = FileDialog::new()
        .add_filter("BGST file", &["bgst3"])
        .pick_file() {
            let data = fs::read(path)?;

            if data.len() == 0 || b"BGST" != &data[..4] {
                bail!("bgst is invalid");
            }

            self.bgst_file = Some(BGSTFile::from_bytes(&data));

            // decode images
            
            let bgst_file = self.bgst_file.as_ref().unwrap();

            self.decoded_image_handles.clear();

            for (index, encoded) in bgst_file.compressed_images.iter().enumerate() {
                let decoded = gctex::decode(
                    encoded,
                    bgst_file.image_width,
                    bgst_file.image_height,
                    gctex::TextureFormat::CMPR,
                    &Vec::new(),
                    0
                );

                let handle = self.get_texture_handle(
                    ui,
                    bgst_file.image_width as usize,
                    bgst_file.image_height as usize,
                    index,
                    &decoded
                );

                self.decoded_image_handles.push(handle);
            }            
            // for entry in bgst_file.bgst_entries.iter() {
            //     if entry.main_image_index > -1 {
            //         let decoded = gctex::decode(
            //             &bgst_file.compressed_images[entry.main_image_index as usize],
            //             bgst_file.image_width,
            //             bgst_file.image_height,
            //             gctex::TextureFormat::CMPR,
            //             &Vec::new(),
            //             0,
            //         );
                    
            //         let handle = self.get_texture_handle(
            //             ui,
            //             bgst_file.image_width as usize,
            //             bgst_file.image_height as usize,
            //             entry.main_image_index as usize,
            //             &decoded
            //         );

            //         self.decoded_image_handles.push(handle);
            //     }

            //     if entry.mask_image_index > -1 {
            //         let decoded = gctex::decode(
            //             &bgst_file.compressed_images[entry.mask_image_index as usize],
            //             bgst_file.image_width,
            //             bgst_file.image_height,
            //             gctex::TextureFormat::I4,
            //             &Vec::new(),
            //             0,
            //         );
                    
            //         let handle = self.get_texture_handle(
            //             ui,
            //             bgst_file.image_width as usize,
            //             bgst_file.image_height as usize,
            //             entry.mask_image_index as usize,
            //             &decoded
            //         );

            //         self.decoded_image_handles.push(handle);
            //     }
            // }
        }

        Ok(())
    }

    fn get_texture_handle(
        &self, 
        ui: &egui::Ui,
        width: usize,
        height: usize,
        index: usize,
        decoded: &[u8]
    ) -> egui::TextureHandle {
        let texture = egui::ColorImage::from_rgba_unmultiplied(
            [width, height],
            &decoded
        );

        ui.ctx().load_texture(format!("le_bgst_image-{}", index), texture, egui::TextureOptions::LINEAR)       
    }

    /// Rendering function for the level editor
    pub fn le_render(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        position: egui::Vec2,
        zoom: f32,
    ) {
        if self.bgst_file.is_none() {
            return;
        }

        // let _x_mult = 1.028;
        // let _y_mult = 1.019;
        // let _image_size = 10.9;

        let bgst_file = self.bgst_file.as_ref().unwrap();
        let painter = ui.painter_at(rect);
        let image_size_vec = self.tile_size * zoom * self.tile_scale;

        // additionally, this offset also seems to
        // align things well
        // let x_offset = -0.8 * 2.0;

        let grid_origin = egui::Vec2::new(
            position.x - self.tile_offset.x,
            position.y - (self.tile_size * zoom * bgst_file.grid_height as f32) - self.tile_offset.y
        );
        
        // ultimately those values aren't perfect, nor do they seem
        // universal, but they handle most cases relatively well


        let num_handles = self.decoded_image_handles.len();
        
        for entry in bgst_file.bgst_entries.iter() {
            if entry.main_image_index > -1 &&
                (entry.main_image_index as usize) < num_handles &&
                entry.enabled
            {
                let tex_handle = &self.decoded_image_handles[entry.main_image_index as usize]; 
                
                let grid_pos = egui::Vec2::new(
                    entry.grid_x_position as f32,
                    entry.grid_y_position as f32,
                );

                let tile_pos = grid_origin + (grid_pos * image_size_vec);

                let tile_rect = egui::Rect::from_min_size(
                    tile_pos.to_pos2(),
                    image_size_vec
                );

                painter.image(
                    tex_handle.id(),
                    tile_rect,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                    egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x20)
                );
            }

            // if entry.mask_image_index > -1 &&
            //     (entry.mask_image_index as usize) < num_handles &&
            //     entry.enabled
            // {
            //     let tex_handle = &self.decoded_image_handles[entry.mask_image_index as usize]; 
                
            //     let grid_pos = egui::Vec2::new(
            //         entry.grid_x_position as f32,
            //         entry.grid_y_position as f32,
            //     );

            //     let tile_pos = grid_origin + (grid_pos * image_size);

            //     let tile_rect = egui::Rect::from_min_size(
            //         tile_pos.to_pos2(),
            //         image_size
            //     );

            //     painter.image(
            //         tex_handle.id(),
            //         tile_rect,
            //         egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
            //         egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x20)
            //     );
            // }
        }
    }
}