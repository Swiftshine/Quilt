use byteorder::{ByteOrder, BigEndian};
use egui::Vec2;
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

    // pub fn to_pos2(&self) -> Pos2 {
    //     Pos2 { x: self.x, y: self.y }
    // }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2 { x: self.x, y: self.y }
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

    pub fn to_point_2d(&self) -> Point2D {
        Point2D {
            x: self.x,
            y: self.y
        }
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

pub struct Camera {
    pub position: egui::Vec2,
    pub zoom: f32
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: egui::Vec2::ZERO,
            zoom: 1.0
        }
    }
}

impl Camera {
    pub fn update(&mut self, ctx: &egui::Context, canvas_response: &egui::Response) {
        let zoom_sensitivity = 0.05;
        let zoom_min = 0.5;
        let zoom_max = 15.0;
    
        // zoom handling
        if canvas_response.hovered() {
            let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_change = zoom_sensitivity * scroll_delta.signum();
                self.zoom = (self.zoom + zoom_change).clamp(zoom_min, zoom_max);
            }
        }
    
        // pan reset handling
        if canvas_response.dragged() && ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.reset();
        }
    }

    pub fn pan(&mut self, delta: egui::Vec2) {
        self.position -= delta;
    }

    pub fn to_camera(&self, pos: egui::Vec2) -> egui::Vec2 {
        egui::Vec2 {
            x: (pos.x - self.position.x) * self.zoom,
            y: (-pos.y - self.position.y) * self.zoom
        }

    }

    // pub fn from_camera(&self, pos: egui::Vec2) -> egui::Vec2 {
    //     egui::Vec2 {
    //         x: (pos.x / self.zoom) + self.position.x,
    //         y: (-pos.y / self.zoom) + self.position.y
    //     }
    // }

    pub fn reset(&mut self) {
        self.position = Default::default();
        self.zoom = 1.0;
    }
}
