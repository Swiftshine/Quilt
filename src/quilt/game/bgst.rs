// use gctex;
// use image;
use byteorder::{ByteOrder, BigEndian};

// const HEADER_SIZE: usize = 0x40;
const GRID_ENTRY_SIZE: usize = 0x10;
const COMPRESSED_IMAGE_SIZE: usize = 0x20000;

#[derive(Default, Clone, Copy)]
// all fields here are i16 in the BGST format
pub struct BGSTEntry {
    pub _enabled: bool,
    pub layer: i16,
    pub grid_x_position: i16,
    pub grid_y_position: i16,
    pub main_image_index: i16,
    pub mask_image_index: i16,
    pub _unk_c: i16,
    pub _unk_e: i16,
}

#[derive(Default)]
pub struct BGSTFile {
    pub _unk_4: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub _grid_width: u32,
    pub grid_height: u32,
    pub _show_layer: [bool; 0xC],
    pub bgst_entries: Vec<BGSTEntry>,
    pub _y_offset: f32,
    pub compressed_images: Vec<Vec<u8>>,
}

impl BGSTFile {
    pub fn from_bytes(input: &[u8]) -> Self {        
        // read header
        let unk_4 = BigEndian::read_u32(&input[4..8]);
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
        let y_offset = BigEndian::read_f32(&input[0x30..0x34]);
        
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
            .map(|img| img.to_vec())
            .collect();

        assert_eq!(image_count, compressed_images.len());

        BGSTFile {
            _unk_4: unk_4,
            image_width,
            image_height,
            _grid_width: grid_width,
            grid_height,
            _show_layer: show_layer,
            bgst_entries,
            _y_offset: y_offset,
            compressed_images
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
            _enabled: enabled,
            layer,
            grid_x_position,
            grid_y_position,
            main_image_index,
            mask_image_index,
            _unk_c,
            _unk_e
        }
    }
}
