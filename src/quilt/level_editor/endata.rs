use byteorder::{ByteOrder, BigEndian};
use crate::quilt::common::*;

// Unlike the mapbin format, the enbin format is not known in its entirety.

const ENEMY_SIZE: usize = 0x174;
const ENEMY_PARAMS_SIZE: usize = 0x18;

#[derive(Default)]
pub struct EnemyParams {
    pub float_params: [f32; 3],
    pub int_params: [i32; 3]
}

#[allow(non_snake_case)]
#[derive(Default)]
pub struct Enemy {
    pub name: String,
    pub behavior: String,
    pub path_name: String,
    pub bead_type: String,
    pub bead_color: String,
    pub direction: String,
    pub unk_88: String,
    pub orientation: String,
    pub position_1: Point3D,
    pub position_2: Point3D,
    pub position_3: Point3D,
    pub params: [EnemyParams; 7],
    pub unk_16C: u32,
    pub unk_170: u32,
}

// #[derive(Default)]
// pub struct Line {
//     pub points: Vec<Point2D>
// }

#[derive(Default)]
pub struct Endata {
    pub enemies: Vec<Enemy>,
    pub unk_footer: Vec<u8>, // these float values dont seem to do anything
}

impl Endata {
    pub fn from_data(input: &[u8]) -> Self {
        let mut endata = Self::default();

        // version? always seems to be 3 in epic yarn and 1 in Wario Land: Shake It!
        // that being said, the enbin format in the latter is similar but not the same here
        // in any case, more research into this format must be done
        // let version = BigEndian::read_u32(&input[4..8]); 

        // always zero
        // let unk_8 = BigEndian::read_u32(&input[8..0xC]);

        // this value is usually 0x14
        let enemy_offset = BigEndian::read_u32(&input[0xC..0x10]) as usize;
        let unk_header_offset = BigEndian::read_u32(&input[0x10..0x14]) as usize;

        let num_enemies = BigEndian::read_u32(&input[enemy_offset..enemy_offset + 4]) as usize;
        for i in 0..num_enemies {
            let start = enemy_offset + 4 + (i * ENEMY_SIZE);
            let end = start + ENEMY_SIZE;
            endata.enemies.push(
                Enemy::from_bytes(
                    &input[start..end]
                )
            );
        }

        endata.unk_footer = input[unk_header_offset..].to_vec();

        endata
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::<u8>::new();

        // header
        out.extend_from_slice(b"GFES");
        out.extend(3u32.to_be_bytes());
        out.extend(0u32.to_be_bytes());
        out.extend(0x14u32.to_be_bytes());
        
        let unk_offset = 0x14 + // header size
            (ENEMY_SIZE * self.enemies.len()) as u32;
        
        
        out.extend(unk_offset.to_be_bytes());

        // enemies
        
        for enemy in self.enemies.iter() {
            out.extend(enemy.to_bytes());
        }

        // the data at the end, if any
        out.extend_from_slice(&self.unk_footer);
        
        out
    }
}


impl EnemyParams {
    fn from_bytes(input: &[u8]) -> Self {
        let mut params = Self::default();

        for i in 0..3 {
            let start = 4 * i;
            let end = start + 4;
            params.float_params[i] = BigEndian::read_f32(&input[start..end]);
        }

        for i in 0..3 {
            let start = 0xC + (4 * i);
            let end = start + 4;
            params.int_params[i] = BigEndian::read_i32(&input[start..end]);
        }

        params
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        for i in 0..3 {
            out.extend(self.float_params[i].to_be_bytes());
        }

        for i in 0..3 {
            out.extend(self.int_params[i].to_be_bytes());
        }

        out
    }

}

impl Enemy {
    fn from_bytes(input: &[u8]) -> Self {
        let mut enemy = Self::default();

        enemy.name = string_from_buffer(&input[..0x20]);
        enemy.behavior = string_from_buffer(&input[0x20..0x40]);
        enemy.path_name = string_from_buffer(&input[0x40..0x60]);
        enemy.bead_type = string_from_buffer(&input[0x60..0x70]);
        enemy.bead_color = string_from_buffer(&input[0x70..0x80]);
        enemy.direction = string_from_buffer(&input[0x80..0x88]);
        enemy.unk_88 = string_from_buffer(&input[0x88..0x90]);
        enemy.orientation = string_from_buffer(&input[0x90..0xA0]);
        enemy.position_1 = Point3D::from_be_bytes(&input[0xA0..0xAC]);
        enemy.position_2 = Point3D::from_be_bytes(&input[0xAC..0xB8]);
        enemy.position_3 = Point3D::from_be_bytes(&input[0xB8..0xC4]);

        for i in 0..7 {
            let start = 0xC4 + (i * ENEMY_PARAMS_SIZE);
            let end = start + ENEMY_PARAMS_SIZE;
            enemy.params[i] = EnemyParams::from_bytes(&input[start..end]);
        }

        enemy.unk_16C = BigEndian::read_u32(&input[0x16C..0x170]);
        enemy.unk_170 = BigEndian::read_u32(&input[0x170..0x174]);

        enemy
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::<u8>::new();

        out.extend(string_to_buffer(&self.name, 0x20));
        out.extend(string_to_buffer(&self.behavior, 0x20));
        out.extend(string_to_buffer(&self.path_name, 0x20));
        out.extend(string_to_buffer(&self.bead_type, 0x10));
        out.extend(string_to_buffer(&self.bead_color, 0x10));
        out.extend(string_to_buffer(&self.direction, 0x08));
        out.extend(string_to_buffer(&self.unk_88, 0x08));
        out.extend(string_to_buffer(&self.orientation, 0x10));

        out.extend(self.position_1.to_be_bytes());
        out.extend(self.position_2.to_be_bytes());
        out.extend(self.position_3.to_be_bytes());

        for param in self.params.iter() {
            out.extend(param.to_bytes());
        }

        out.extend(&self.unk_16C.to_be_bytes());
        out.extend(&self.unk_170.to_be_bytes());

        out
    }
}



// impl Line {
//     fn from_bytes(input: &[u8], count: usize) -> Self {
//         let mut line = Self::default();

//         for i in 0..count {
//             let start = i * 4;
//             let end = start + 8;
//             line.points.push(
//                 Point2D::from_be_bytes(&input[start..end])
//             );
//         };

//         line
//     }

//     // fn to_bytes(&self) -> Vec<u8> {
//     //     todo!()
//     // }
// }
