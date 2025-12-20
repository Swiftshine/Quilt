use gctex;
use image::{self, GenericImageView};
// use std::{collections::{HashMap, HashSet}, fs};
use std::fs;

use byteorder::{BigEndian, ByteOrder};

use anyhow::{Result, bail};

const HEADER_SIZE: usize = 0x40;
const GRID_ENTRY_SIZE: usize = 0x10;
const COMPRESSED_IMAGE_SIZE: usize = 0x20000;
const BGST_TILE_SIZE: u32 = 512; // in extra epic yarn, this is 256. todo: check if this can safely be changed

pub const LAYER_NAMES: [&str; 12] = [
    "Far 5", "Far 4", "Far 3", "Far 2", "Far 1", "Map", "Game", "Near 1", "Near 2", "Near 3",
    "Near 4", "Near 5",
];

#[derive(Default, Clone, Copy, Debug)]
// all fields here are i16 in the BGST format
pub struct BGSTEntry {
    pub enabled: bool,
    pub layer: i16,
    pub grid_x_position: i16,
    pub grid_y_position: i16,
    pub main_image_index: i16,
    pub mask_image_index: i16,
    pub _unk_c: i16,
    pub _unk_e: i16,
}

// helper methods
impl BGSTEntry {
    pub fn main_valid(&self) -> bool {
        self.main_image_index > -1
    }

    pub fn mask_valid(&self) -> bool {
        self.mask_image_index > -1
    }

    pub fn is_masked(&self) -> bool {
        self.main_valid() && self.mask_valid()
    }

    pub fn is_valid(&self) -> bool {
        self.main_valid() || self.mask_valid()
    }
}

#[derive(Default, Debug)]
pub struct BGSTFile {
    pub flags: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub _show_layer: [bool; 0xC],
    pub bgst_entries: Vec<BGSTEntry>,
    pub scale_modifier: f32,
    pub compressed_images: Vec<Vec<u8>>,
}

impl BGSTFile {
    pub fn from_bytes(input: &[u8]) -> Self {
        // read header
        let flags = BigEndian::read_u32(&input[4..8]);
        let image_width = BigEndian::read_u32(&input[8..0xC]);
        let image_height = BigEndian::read_u32(&input[0xC..0x10]);
        let grid_width = BigEndian::read_u32(&input[0x10..0x14]);
        let grid_height = BigEndian::read_u32(&input[0x14..0x18]);

        let image_count = BigEndian::read_u32(&input[0x18..0x1C]) as usize;

        let mut show_layer: [bool; 0xC] = [false; 0xC];

        for i in 0..0xC {
            show_layer[i] = input[0x1C + i] != 0;
        }

        let info_offset = BigEndian::read_u32(&input[0x28..0x2C]) as usize;
        let image_data_offset = BigEndian::read_u32(&input[0x2C..0x30]) as usize;
        let scale_modifier = BigEndian::read_f32(&input[0x30..0x34]);

        // read entries
        let mut current_offset = info_offset;

        let mut bgst_entries = Vec::new();

        while current_offset < image_data_offset {
            let start = current_offset;
            let end = current_offset + GRID_ENTRY_SIZE;
            let entry = BGSTEntry::from_bytes(&input[start..end]);
            bgst_entries.push(entry);
            current_offset += GRID_ENTRY_SIZE;
        }

        // read compressed images
        let compressed_images: Vec<Vec<u8>> = input[image_data_offset..]
            .chunks(COMPRESSED_IMAGE_SIZE)
            .take(image_count) // don't read padding
            .map(|img| img.to_vec())
            .collect();

        for img in compressed_images.iter() {
            assert_eq!(img.len(), COMPRESSED_IMAGE_SIZE);
        }

        assert_eq!(image_count, compressed_images.len());

        BGSTFile {
            flags,
            image_width,
            image_height,
            grid_width,
            grid_height,
            _show_layer: show_layer,
            bgst_entries,
            scale_modifier,
            compressed_images,
        }
    }

    // /// Removes unused entries and images
    // ! Don't use yet
    // pub fn cleanup(&mut self) {
    //     // collect used indices
    //     let mut used_indices = HashSet::new();

    //     let valid_entries: Vec<&BGSTEntry> = self.bgst_entries.iter()
    //     .filter(|entry| entry.is_valid()).collect();

    //     for entry in valid_entries.iter() {
    //         if entry.main_valid() {
    //             used_indices.insert(entry.main_image_index as usize);
    //         }

    //         if entry.mask_valid() {
    //             used_indices.insert(entry.mask_image_index as usize);
    //         }
    //     }

    //     // (old, new)
    //     let mut old_to_new_index: HashMap<usize, usize> = HashMap::new();
    //     let mut new_compressed_images = Vec::new();
    //     let mut new_index_counter = 0;

    //     for (old_index, compressed_data) in self.compressed_images.drain(..).enumerate() {
    //         if used_indices.contains(&old_index) {
    //             // keep the image
    //             new_compressed_images.push(compressed_data);

    //             // store mapping
    //             old_to_new_index.insert(old_index, new_index_counter);
    //             new_index_counter += 1;
    //         } // otherwise it's discarded
    //     }

    //     self.compressed_images = new_compressed_images;

    //     // update BGST entry indices

    //     for entry in self.bgst_entries.iter_mut() {
    //         if entry.main_valid() {
    //             let old_index = entry.main_image_index as usize;

    //             if let Some(&new_index) = old_to_new_index.get(&old_index) {
    //                 entry.main_image_index = new_index as i16;
    //             } else {
    //                 entry.main_image_index = -1;
    //             }
    //         }

    //         if entry.mask_valid() {
    //             let old_index = entry.mask_image_index as usize;

    //             if let Some(&new_index) = old_to_new_index.get(&old_index) {
    //                 entry.mask_image_index = new_index as i16;
    //             } else {
    //                 entry.mask_image_index = -1;
    //             }
    //         }
    //     }

    //     // remove redundant entries
    //     self.bgst_entries.retain(|e| e.is_valid());
    // }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        // header
        out.extend(String::from("BGST").as_bytes());
        out.extend(self.flags.to_be_bytes());
        out.extend(self.image_width.to_be_bytes());
        out.extend(self.image_height.to_be_bytes());
        out.extend(self.grid_width.to_be_bytes());
        out.extend(self.grid_height.to_be_bytes());

        let image_count = self.compressed_images.len() as u32;

        out.extend(image_count.to_be_bytes());

        for layer in self._show_layer {
            out.push(layer as u8);
        }

        out.extend((HEADER_SIZE as u32).to_be_bytes());
        let image_data_offset = (HEADER_SIZE + (self.bgst_entries.len() * GRID_ENTRY_SIZE)) as u32;

        out.extend(image_data_offset.to_be_bytes());
        out.extend(self.scale_modifier.to_be_bytes());
        out.resize(out.len() + 0xC, 0); // padding

        // entries

        for entry in self.bgst_entries.iter() {
            out.extend((entry.enabled as i16).to_be_bytes());
            out.extend(entry.layer.to_be_bytes());

            out.extend(entry.grid_x_position.to_be_bytes());
            out.extend(entry.grid_y_position.to_be_bytes());

            out.extend(entry.main_image_index.to_be_bytes());
            out.extend(entry.mask_image_index.to_be_bytes());

            out.extend(entry._unk_c.to_be_bytes());
            out.extend(entry._unk_e.to_be_bytes());
        }

        // compressed chunks

        for chunk in self.compressed_images.iter() {
            out.extend_from_slice(chunk);
        }

        // padding
        out.resize(out.len().next_multiple_of(0x20), 0);

        out
    }

    pub fn remove_entry(&mut self, entry_index: usize) {
        self.bgst_entries.remove(entry_index);
    }

    /// Creates a new BGST entry and associated image
    pub fn create_entry(
        &mut self,
        layer: i16,
        (x, y): (i16, i16), // (x, y)
    ) -> Result<()> {
        // you'd have to have a main image before applying a mask
        self.add_image(gctex::TextureFormat::CMPR)?;

        let entry = BGSTEntry {
            enabled: true,
            layer,
            grid_x_position: x,
            grid_y_position: y,
            main_image_index: (self.compressed_images.len() - 1) as i16,
            mask_image_index: -1,
            _unk_c: -1,
            _unk_e: -1,
        };

        self.bgst_entries.push(entry);

        Ok(())
    }

    /// Uses a file dialog to replace an image.
    pub fn replace_image(
        &mut self,
        image_index: Option<usize>,
        format: gctex::TextureFormat,
    ) -> Result<()> {
        // get path

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image file", &["png"]) // just png for now
            .pick_file()
        {
            let file_data = fs::read(path)?;

            if file_data.is_empty() {
                bail!("empty file");
            }

            // decode image

            let img = match image::load_from_memory_with_format(&file_data, image::ImageFormat::Png)
            {
                Ok(img) => img,
                Err(e) => bail!("failed to decode image: {}", e),
            };

            let (width, height) = img.dimensions();

            if width != BGST_TILE_SIZE || height != BGST_TILE_SIZE {
                bail!(
                    "image dimensions must be {}x{}",
                    BGST_TILE_SIZE,
                    BGST_TILE_SIZE
                );
            };

            // get raw rgba
            let rgba = img.into_rgba8().into_raw();

            // again this is epic yarn (wii) only for now

            let compressed = gctex::encode(format, &rgba, BGST_TILE_SIZE, BGST_TILE_SIZE);

            if let Some(image_index) = image_index {
                // replacing an existing one
                self.compressed_images[image_index] = compressed;
            } else {
                // adding a new one
                self.compressed_images.push(compressed);
            }

            Ok(())
        } else {
            bail!("User exited")
        }
    }

    /// Removes an entry's mask image.
    /// ### Returns
    /// Whether or not an image was removed.
    pub fn remove_entry_mask(&mut self, entry_index: usize) -> bool {
        let mask_index = self.bgst_entries[entry_index].mask_image_index;
        self.bgst_entries[entry_index].mask_image_index = -1;
        
        // check if there are any other users of this mask
        let num_users = self.bgst_entries
            .iter()
            .enumerate()
            .filter(|(i, e)| *i != entry_index && e.mask_image_index == mask_index)
            .count();
        
        if num_users == 0 { // we can remove the image
            // account for every entry with a mask index greater than the existing one
            self.compressed_images.remove(mask_index as usize);
    
            for entry in self.bgst_entries.iter_mut() {
                if entry.main_image_index > mask_index {
                    entry.main_image_index -= 1;
                }
    
                if entry.mask_image_index > mask_index {
                    entry.mask_image_index -= 1;
                }
            }

            true
        } else { false }
    }

    /// Uses a file dialog to add an image.
    /// ### Returns
    /// The index of the compressed image.
    pub fn add_image(&mut self, format: gctex::TextureFormat) -> Result<usize> {
        self.replace_image(None, format)?;
        Ok(self.compressed_images.len() - 1)
    }

    pub fn export_image(&self, image_index: usize, format: gctex::TextureFormat) -> Result<()> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image file", &["png"])
            .save_file()
        {
            let decompressed = gctex::decode(
                &self.compressed_images[image_index],
                BGST_TILE_SIZE,
                BGST_TILE_SIZE,
                format,
                &Vec::new(),
                0,
            );

            image::save_buffer(
                path,
                &decompressed,
                BGST_TILE_SIZE,
                BGST_TILE_SIZE,
                image::ExtendedColorType::Rgba8,
            )?;

            Ok(())
        } else {
            bail!("user exited")
        }
    }
}

impl BGSTEntry {
    pub fn from_bytes(input: &[u8]) -> Self {
        let enabled = BigEndian::read_i16(&input[..2]) != 0;
        let layer = BigEndian::read_i16(&input[2..4]);
        let grid_x_position = BigEndian::read_i16(&input[4..6]);
        let grid_y_position = BigEndian::read_i16(&input[6..8]);
        let main_image_index = BigEndian::read_i16(&input[8..0xA]);
        let mask_image_index = BigEndian::read_i16(&input[0xA..0xC]);
        let _unk_c = BigEndian::read_i16(&input[0xC..0xE]);
        let _unk_e = BigEndian::read_i16(&input[0xE..0x10]);

        BGSTEntry {
            enabled,
            layer,
            grid_x_position,
            grid_y_position,
            main_image_index,
            mask_image_index,
            _unk_c,
            _unk_e,
        }
    }
}
