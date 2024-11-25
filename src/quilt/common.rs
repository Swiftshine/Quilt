use byteorder::{ByteOrder, BigEndian};
use encoding_rs::SHIFT_JIS;

pub fn shift_jis_to_utf8(raw: &[u8]) -> String {
    let (decoded, _, has_errors) = SHIFT_JIS.decode(raw);

    if has_errors {
        return String::from("<DECODE ERROR>");
    }

    decoded.to_string()
}

#[derive(Default)]
pub struct Point2D {
    pub x: f32,
    pub y: f32
}

impl Point2D {
    pub fn from_be_bytes(input: &[u8]) -> Self {
        let mut point = Self::default();

        point.x = BigEndian::read_f32(&input[..4]);
        point.y = BigEndian::read_f32(&input[4..8]);

        point
    }
}

#[derive(Default)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Point3D {
    pub fn from_be_bytes(input: &[u8]) -> Self {
        let mut point = Self::default();
    
        point.x = BigEndian::read_f32(&input[..4]);
        point.y = BigEndian::read_f32(&input[4..8]);
        point.z = BigEndian::read_f32(&input[8..0xC]);

        point
    }
}
#[derive(Default)]
pub struct NameMap {
    pub names: Vec<String>, // ShiftJIS or ASCII names
    pub indices: Vec<usize>,
}

pub fn string_from_buffer(input: &[u8]) -> String {
    let null_terminator_pos = input[..]
        .iter()
        .position(|&byte| byte == 0x00)
        .unwrap_or(input.len());

    let string = &input[..null_terminator_pos];

    String::from_utf8(string.to_vec()).unwrap_or_else(|_| String::default())
}

impl NameMap {
    pub fn read_names(
        &mut self,
        input: &[u8],
        count: usize,
        size: usize,
        start_offset: usize // offset to the "footer"
    ) {
        for i in 0..count {
            let start = start_offset + (i * size);
            let end = start + size;

            let name_bytes = &input[start..end];

            let null_terminator_pos = name_bytes
                .iter()
                .position(|&byte| byte == 0x00)
                .unwrap_or(size);

            let name = &name_bytes[..null_terminator_pos];

            self.indices.push(i);
            self.names.push(shift_jis_to_utf8(&name));
        }
    }
}
