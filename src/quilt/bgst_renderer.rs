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
    pub opacity: u8,
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
            opacity: 128,
            
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


            // determine which entries have masks
            let (masked, _): (Vec<BGSTEntry>, Vec<_>) = bgst_file
            .bgst_entries
            .iter()
            .partition(|entry| entry.main_image_index > -1 && entry.mask_image_index > -1);

            let num_images = self.decoded_image_handles.len();

            // cache the masked entries
            for entry in masked.iter() {
                let main_index = entry.main_image_index as usize;
                let mask_index = entry.mask_image_index as usize;

                if main_index as usize >= num_images || mask_index as usize >= num_images {
                    // they're invalid
                    continue;
                }

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

        let bgst_file = self.bgst_file.as_ref().unwrap();
        let painter = ui.painter_at(rect);
        let image_size_vec = self.tile_size * zoom * self.tile_scale;

        // collect entries based on whether or not
        // a mask is applied

        let (mut masked, mut unmasked): (Vec<BGSTEntry>, Vec<BGSTEntry>) = bgst_file
            .bgst_entries
            .iter()
            .partition(|entry| entry.main_image_index > -1 && entry.mask_image_index > -1);


        // sort both vectors by entry layer
        masked.sort_by(|a, b| a.layer.cmp(&b.layer));
        unmasked.sort_by(|a, b| a.layer.cmp(&b.layer));

        let grid_origin = egui::Vec2::new(
            position.x - self.tile_offset.x,
            position.y - (self.tile_size * zoom * bgst_file.grid_height as f32) - self.tile_offset.y
        );


        let num_handles = self.decoded_image_handles.len();
        
        // render unmasked
        for entry in unmasked.iter() {
            // get entry that isn't invalid
            let index = std::cmp::max(entry.main_image_index, entry.mask_image_index);

            if index < 0 || index as usize >= num_handles {
                // both are invalid
                continue;
            }
            
            let tex_handle = &self.decoded_image_handles[index as usize];
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
                egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, self.opacity)
            );
        }
        
        // render masked
        for entry in masked.iter() {
            let main_index = entry.main_image_index as usize;
            let mask_index = entry.mask_image_index as usize;
            
            let masked_texture = self.masked_textures.get(&(main_index, mask_index));
            
            let grid_pos = egui::Vec2::new(
                entry.grid_x_position as f32,
                entry.grid_y_position as f32,
            );
            
            let tile_pos = grid_origin + (grid_pos * image_size_vec);
            
            let tile_rect = egui::Rect::from_min_size(
                tile_pos.to_pos2(),
                image_size_vec
            );
            
            if let Some(tex) = masked_texture {
                painter.image(
                    tex.id(),
                    tile_rect,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                    egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, self.opacity)
                );
            }
        }
    }
}
