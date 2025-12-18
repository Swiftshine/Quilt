use crate::quilt::game::bgst::*;
use anyhow::{Result, anyhow, bail};
use egui::TextureOptions;
use image::{ImageBuffer, RgbaImage};
use rayon::prelude::*;
use rfd::FileDialog;
use std::{collections::HashMap, fs, path::PathBuf};

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
    pub zoom: f32,
}

impl BGSTRenderer {
    pub fn apply_mask(target: &[u8], mask: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
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

    pub fn get_raw_image_by_texture_handle(
        &self,
        tex_handle: &egui::TextureHandle,
    ) -> Result<Vec<u8>> {
        let handle_id = tex_handle.id();

        if let Some(image_data) = self.raw_image_data.get(&handle_id) {
            Ok(image_data.clone())
        } else {
            bail!(
                "no raw image data found for texture handle id {:?}",
                handle_id
            );
        }
    }

    pub fn new() -> Self {
        Self {
            tile_size: 11.9,
            // tile_size: 10.0,
            tile_scale: egui::Vec2::new(1.028, 1.019),
            // tile_scale: egui::Vec2::splat(1.0),
            tile_offset: egui::Vec2::splat(0.0),
            opacity: 128,
            zoom: 1.0,

            ..Default::default()
        }
    }

    pub fn open_file(&mut self, ui: &egui::Ui) -> Result<PathBuf> {
        if let Some(path) = FileDialog::new()
            .add_filter("BGST file", &["bgst3"])
            .pick_file()
        {
            let data = fs::read(&path)?;

            if data.is_empty() || b"BGST" != &data[..4] {
                bail!("bgst is invalid");
            }

            self.bgst_file = Some(BGSTFile::from_bytes(&data));
            let _ = self.cache_textures(ui.ctx());

            Ok(path)
        } else {
            bail!("user exited")
        }
    }

    /// Decodes and caches images
    pub fn cache_textures(&mut self, ctx: &egui::Context) -> Result<()> {
        // put this here
        let bgst_file = match self.bgst_file.as_ref() {
            Some(file) => file,
            None => bail!("BGST file not loaded")
        };

        // clear old data
        self.decoded_image_handles.clear();
        self.masked_textures.clear();
        self.raw_image_data.clear();

        let results: Vec<(egui::TextureHandle, Vec<u8>)> = bgst_file
            .compressed_images
            .par_iter()
            .enumerate()
            .map(|(index, encoded)| {
                let tex_format = {
                    let is_main = bgst_file.bgst_entries.iter().any(|entry| {
                        entry.main_image_index > -1 && entry.main_image_index as usize == index
                    });
                    let is_mask = bgst_file.bgst_entries.iter().any(|entry| {
                        entry.mask_image_index > -1 && entry.mask_image_index as usize == index
                    });
                    match (is_main, is_mask) {
                        (true, _) => gctex::TextureFormat::CMPR, // "main" image
                        (_, true) => gctex::TextureFormat::I4,   // mask image
                        _ => gctex::TextureFormat::CMPR,         // default
                    }
                };
                let decoded = gctex::decode(
                    encoded,
                    bgst_file.image_width,
                    bgst_file.image_height,
                    tex_format,
                    &Vec::new(),
                    0,
                );
                let handle = self.load_texture_handle(
                    ctx,
                    bgst_file.image_width as usize,
                    bgst_file.image_height as usize,
                    index,
                    &decoded,
                );
                (handle, decoded)
            })
        .collect();

        for (handle, decoded) in results {
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
        let masked_results: Vec<((usize, usize), egui::TextureHandle)> = masked
            .into_par_iter()
            .filter_map(|entry| {
                let main_index = entry.main_image_index as usize;
                let mask_index = entry.mask_image_index as usize;
                if main_index >= num_images || mask_index >= num_images {
                    // they're invalid
                    return None;
                }
                let main_handle = &self.decoded_image_handles[main_index];
                let mask_handle = &self.decoded_image_handles[mask_index];
                let main_image = match self.get_raw_image_by_texture_handle(main_handle) {
                    Ok(img) => img,
                    Err(_) => return None,
                };
                let mask_image = match self.get_raw_image_by_texture_handle(mask_handle) {
                    Ok(img) => img,
                    Err(_) => return None,
                };
                // apply masks
                let masked_image = match BGSTRenderer::apply_mask(
                    &main_image,
                    &mask_image,
                    bgst_file.image_width,
                    bgst_file.image_height,
                ) {
                    Ok(img) => img,
                    Err(_) => return None,
                };
                // load textures
                let masked_texture = ctx.load_texture(
                    format!("be_masked_tex_{}-{}", main_index, mask_index),
                    egui::ColorImage::from_rgba_unmultiplied(
                        [
                            bgst_file.image_width as usize,
                            bgst_file.image_height as usize,
                        ],
                        &masked_image,
                    ),
                    TextureOptions::LINEAR,
                );
                Some(((main_index, mask_index), masked_texture))
        })
        .collect();

        for ((main_index, mask_index), masked_texture) in masked_results {
            self.masked_textures
                .insert((main_index, mask_index), masked_texture);
        }

        Ok(())
    }

    fn load_texture_handle(
        &self,
        ctx: &egui::Context,
        width: usize,
        height: usize,
        index: usize,
        decoded: &[u8],
    ) -> egui::TextureHandle {
        let texture = egui::ColorImage::from_rgba_unmultiplied([width, height], decoded);

        ctx.load_texture(
            format!("le_bgst_image-{}", index),
            texture,
            egui::TextureOptions::LINEAR,
        )
    }

    pub fn get_texture_handle(&self, index: usize) -> Option<&egui::TextureHandle> {
        if let Some(bgst_file) = self.bgst_file.as_ref() {
            let entry = &bgst_file.bgst_entries[index];
    
            let main_index = entry.main_image_index as usize;
            let mask_index = entry.mask_image_index as usize;

            
            if entry.is_masked() {
                Some(self.masked_textures.get(&(main_index, mask_index)).unwrap())
            } else {
                Some(&self.decoded_image_handles[main_index])
            }
        } else {
            None
        }
    }
    /// Rendering function for the level editor
    pub fn le_render(&self, ui: &mut egui::Ui, rect: egui::Rect, position: egui::Vec2) {
        if self.bgst_file.is_none() {
            return;
        }

        let bgst_file = self.bgst_file.as_ref().unwrap();

        // collect entries based on whether or not
        // a mask is applied

        let (mut masked, mut unmasked): (Vec<BGSTEntry>, Vec<BGSTEntry>) = bgst_file
            .bgst_entries
            .iter()
            .partition(|entry| entry.is_masked());

        // sort both vectors by entry layer
        masked.sort_by(|a, b| a.layer.cmp(&b.layer));
        unmasked.sort_by(|a, b| a.layer.cmp(&b.layer));

        let grid_origin = egui::Vec2::new(
            position.x - self.tile_offset.x,
            position.y
                - (self.tile_size * self.zoom * bgst_file.grid_height as f32)
                - self.tile_offset.y,
        );

        // println!("BGSTRenderer::le_render - grid_origin: {}", grid_origin);

        // render unmasked
        for entry in unmasked.iter() {
            self.render_unmasked_entry(ui, rect, entry, grid_origin);
        }

        // render masked
        for entry in masked.iter() {
            self.render_masked_entry(ui, rect, entry, grid_origin);
        }
    }

    pub fn render_unmasked_entry(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        entry: &BGSTEntry,
        origin: egui::Vec2,
    ) {
        let num_handles = self.decoded_image_handles.len();

        let index = std::cmp::max(entry.main_image_index, entry.mask_image_index);

        if index < 0 || index as usize >= num_handles {
            return;
        }

        let tex_handle = &self.decoded_image_handles[index as usize];
        let grid_pos = egui::Vec2::new(entry.grid_x_position as f32, entry.grid_y_position as f32);

        let image_size_vec = self.tile_size * self.zoom * self.tile_scale;
        let tile_pos = origin + (grid_pos * image_size_vec);

        let tile_rect = egui::Rect::from_min_size(tile_pos.to_pos2(), image_size_vec);

        let painter = ui.painter_at(rect);

        painter.image(
            tex_handle.id(),
            tile_rect,
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
            egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, self.opacity),
        );
    }

    pub fn render_masked_entry(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        entry: &BGSTEntry,
        origin: egui::Vec2,
    ) {
        let painter = ui.painter_at(rect);

        let main_index = entry.main_image_index as usize;
        let mask_index = entry.mask_image_index as usize;

        let masked_texture = self.masked_textures.get(&(main_index, mask_index));
        let grid_pos = egui::Vec2::new(entry.grid_x_position as f32, entry.grid_y_position as f32);

        let image_size_vec = self.tile_size * self.zoom * self.tile_scale;

        let tile_pos = origin + (grid_pos * image_size_vec);

        let tile_rect = egui::Rect::from_min_size(tile_pos.to_pos2(), image_size_vec);

        if let Some(tex) = masked_texture {
            painter.image(
                tex.id(),
                tile_rect,
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, self.opacity),
            );
        }
    }
}
