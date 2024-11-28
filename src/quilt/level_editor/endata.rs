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

#[derive(Default)]
pub struct Line {
    pub points: Vec<Point2D>
}

#[derive(Default)]
pub struct Endata {
    pub enemies: Vec<Enemy>,
    pub lines: Vec<Line>,
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
        let line_header_offset = BigEndian::read_u32(&input[0x10..0x14]) as usize;

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

        // let num_line_entries = BigEndian::read_u32(&input[line_header_offset..line_header_offset + 4]);
        // let line_offset = BigEndian::read_u32(&input[line_header_offset + 4..line_header_offset + 8]) as usize;
        
        // let mut line_offset_total = 0;
        // for _ in 0..num_line_entries {
        //     let start = line_offset + 4 + line_offset_total;

        //     let line = Line::from_bytes(&input[start..]);

        //     line_offset_total += 4 + (8 * line.points.len());

        //     endata.lines.push(line);
        // }

        let num_lines = BigEndian::read_u32(&input[line_header_offset..line_header_offset + 4]) as usize;
        let mut total_points = 0;
        for i in 0..num_lines {
            let start = line_header_offset + 8 + (total_points * 8) + (i * 4);

            let count = BigEndian::read_u32(&input[start..start + 4]) as usize;
            
            endata.lines.push(
                Line::from_bytes(
                    &input[start..],
                    count
                )
            );

            total_points += count;
        }
        
        endata
    }

    // pub fn to_data(&self) -> Vec<u8> {
    //     todo!()
    // }
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

    // fn to_bytes(&self) -> Vec<u8> {
    //     todo!()
    // }
}



impl Line {
    fn from_bytes(input: &[u8], count: usize) -> Self {
        let mut line = Self::default();

        for i in 0..count {
            let start = i * 4;
            let end = start + 8;
            line.points.push(
                Point2D::from_be_bytes(&input[start..end])
            );
        };

        line
    }

    // fn to_bytes(&self) -> Vec<u8> {
    //     todo!()
    // }
}
