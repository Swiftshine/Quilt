use anyhow::{anyhow, bail, Result};
use egui::TextureOptions;
use image::{ImageBuffer, RgbaImage};
use rfd::FileDialog;
use std::{collections::HashMap, fs};

use crate::quilt::game::bgst::*;


#[derive(Default)]
pub struct BGSTRenderer {
    pub bgst_file: Option<BGSTFile>,
    pub decoded_image_handles: Vec<egui::TextureHandle>,
    pub raw_image_data: HashMap<egui::TextureId, Vec<u8>>,
    pub masked_textures: HashMap<(usize, usize), egui::TextureHandle>,
    pub tile_size: f32,
    pub tile_offset: egui::Vec2,
    pub tile_scale: egui::Vec2,
}


impl BGSTRenderer {
    pub fn apply_mask(
        target: &[u8],
        mask: &[u8],
        width: u32,
        height: u32
    ) -> Result<Vec<u8>> {
        if mask.len() != mask.len() {
            bail!("the image sizes are not equal");
        }

        let target_image: RgbaImage = ImageBuffer::from_raw(width, height, target.to_vec())
            .ok_or_else(|| anyhow!("failed to decode target image"))?;
        
        let masked_image: RgbaImage = ImageBuffer::from_raw(width, height, mask.to_vec())
            .ok_or_else(|| anyhow!("failed to decode mask image"))?;

        let mut output_image = RgbaImage::new(width, height);

        for (x, y, pixel) in output_image.enumerate_pixels_mut() {
            let target_pixel = target_image.get_pixel(x, y);
            let mask_pixel = masked_image.get_pixel(x, y);

            // if the mask pixel is black, set alpha of main image to 0
            if mask_pixel[0] == 0 && mask_pixel[1] == 0 && mask_pixel[2] == 0 {
                *pixel = image::Rgba([target_pixel[0], target_pixel[1], target_pixel[2], 0]); // make transparent
            } else {
                *pixel = *target_pixel;
            }
        }

        let output_bytes = output_image.into_raw();

        Ok(output_bytes)
    }

    pub fn get_raw_image_by_texture_handle(&self, tex_handle: &egui::TextureHandle) -> Result<Vec<u8>> {
        let handle_id = tex_handle.id();

        if let Some(image_data) = self.raw_image_data.get(&handle_id) {
            Ok(image_data.clone())
        } else {
            bail!("no raw image data found for texture handle id {:?}", handle_id);
        }
    }


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
            self.masked_textures.clear();

            for (index, encoded) in bgst_file.compressed_images.iter().enumerate() {
                // determine how to handle the texture
                let tex_format = {
                    if let Some(_) =
                    bgst_file.bgst_entries.iter()
                    .position(|entry| entry.main_image_index > -1 && entry.main_image_index as usize == index)
                    {
                        gctex::TextureFormat::CMPR
                    } else if let Some(_) =
                    bgst_file.bgst_entries.iter()
                    .position(|entry| entry.mask_image_index > -1 && entry.mask_image_index as usize == index)
                    {
                        gctex::TextureFormat::I4
                    } else {
                        gctex::TextureFormat::CMPR
                    }
                };

                let decoded = gctex::decode(
                    encoded,
                    bgst_file.image_width,
                    bgst_file.image_height,
                    tex_format,
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

                self.raw_image_data.insert(handle.id(), decoded);
                self.decoded_image_handles.push(handle);
            }


            // determine whichi entries have masks
            let (masked, _): (Vec<BGSTEntry>, Vec<_>) = bgst_file
            .bgst_entries
            .iter()
            .partition(|entry| entry.main_image_index > -1 && entry.mask_image_index > -1);

            // cache the masked entries
            for entry in masked.iter() {
                let main_index = entry.main_image_index as usize;
                let mask_index = entry.mask_image_index as usize;

                let main_handle = &self.decoded_image_handles[main_index];
                let mask_handle = &self.decoded_image_handles[mask_index];

                let main_image = self.get_raw_image_by_texture_handle(main_handle)?;
                let mask_image = self.get_raw_image_by_texture_handle(mask_handle)?;

                let masked_image = BGSTRenderer::apply_mask(
                    &main_image,
                    &mask_image,
                    bgst_file.image_width,
                    bgst_file.image_height
                )?;

                let masked_texture = ui.ctx().load_texture(
                    format!("be_masked_tex_{}-{}", main_index, mask_index),
                    egui::ColorImage::from_rgba_unmultiplied(
                        [bgst_file.image_width as usize, bgst_file.image_height as usize],
                        &masked_image
                    ),
                    TextureOptions::LINEAR
                );

                self.masked_textures.insert((main_index, mask_index), masked_texture);
            }
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
