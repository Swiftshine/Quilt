// // use gctex;
// // use image;
// use byteorder::{ByteOrder, BigEndian};

// // const HEADER_SIZE: usize = 0x40;
// const GRID_ENTRY_SIZE: usize = 0x10;
// const COMPRESSED_IMAGE_SIZE: usize = 0x20000;

// #[derive(Default, Clone, Copy)]
// // all fields here are i16 in the BGST format
// pub struct BGSTEntry {
//     pub enabled: bool,
//     pub layer: i16,
//     pub grid_x_position: i16,
//     pub grid_y_position: i16,
//     pub main_image_index: i16,
//     pub mask_image_index: i16,
//     pub _unk_c: i16,
//     pub _unk_e: i16,
// }

// #[derive(Default)]
// pub struct BGSTFile {
//     pub unk_4: u32,
//     pub image_width: u32,
//     pub image_height: u32,
//     pub grid_width: u32,
//     pub grid_height: u32,
//     pub show_layer: [bool; 0xC],
//     pub bgst_entries: Vec<BGSTEntry>,
//     pub y_offset: f32,
//     pub compressed_images: Vec<Vec<u8>>,
// }

// impl BGSTFile {
//     pub fn from_bytes(input: &[u8]) -> Self {
//         let mut bgst_file = BGSTFile::default();
        
//         // read header
//         bgst_file.unk_4 = BigEndian::read_u32(&input[4..8]);
//         bgst_file.image_width = BigEndian::read_u32(&input[8..0xC]);
//         bgst_file.image_height = BigEndian::read_u32(&input[0xC..0x10]);
//         bgst_file.grid_width = BigEndian::read_u32(&input[0x10..0x14]);
//         bgst_file.grid_height = BigEndian::read_u32(&input[0x14..0x18]);

//         let image_count = BigEndian::read_u32(&input[0x18..0x1C]) as usize;
        
//         for i in 0..0xC {
//             bgst_file.show_layer[i] = input[0x1C + i] != 0;
//         }

//         let info_offset = BigEndian::read_u32(&input[0x28..0x2C]) as usize;
//         let image_data_offset = BigEndian::read_u32(&input[0x2C..0x30]) as usize;
//         bgst_file.y_offset = BigEndian::read_f32(&input[0x30..0x34]);
        
//         // read entries
//         let mut current_offset = info_offset;
        
//         while current_offset < image_data_offset {
//             let start = current_offset;
//             let end = current_offset + GRID_ENTRY_SIZE;
//             let entry = BGSTEntry::from_bytes(&input[start..end]);
//             bgst_file.bgst_entries.push(entry);
//             current_offset += GRID_ENTRY_SIZE;
//         }

//         // read compressed images
//         bgst_file.compressed_images.extend(
//             input[image_data_offset..]
//             .chunks(COMPRESSED_IMAGE_SIZE)
//             .map(|img| img.to_vec())
//         );

//         assert_eq!(image_count, bgst_file.compressed_images.len());

//         bgst_file
//     }
// }

// impl BGSTEntry {
//     pub fn from_bytes(input: &[u8]) -> Self {
//         let mut entry = Self::default();

//         entry.enabled = BigEndian::read_i16(&input[..2]) != 0;
//         entry.layer = BigEndian::read_i16(&input[2..4]);
//         entry.grid_x_position = BigEndian::read_i16(&input[4..6]);
//         entry.grid_y_position = BigEndian::read_i16(&input[6..8]);
//         entry.main_image_index = BigEndian::read_i16(&input[8..0xA]);
//         entry.mask_image_index = BigEndian::read_i16(&input[0xA..0xC]);
//         entry._unk_c = BigEndian::read_i16(&input[0xC..0xE]);
//         entry._unk_e = BigEndian::read_i16(&input[0xE..0x10]);
        
//         entry
//     }
// }
