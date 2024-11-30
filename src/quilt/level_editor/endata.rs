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


    pub is_selected: bool
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
            4 + // enemy count
            (ENEMY_SIZE * self.enemies.len()) as u32; // enemy entries
        
        
        out.extend(unk_offset.to_be_bytes());
        out.extend((self.enemies.len() as u32).to_be_bytes());

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


pub fn color_string_to_label(s: &str) -> String {
    // the color "PURPLE" is typed as "PERPLE" in the game's code.

    match s {
        "RED" => "Red",
        "ORANGE" => "Orange",
        "YELLOW" => "Yellow",
        "GREEN" => "Green",
        "BLUE" => "Blue",
        "PERPLE" => "Purple",
        "WHITE" => "White",
        "RANDOM" => "Random",
        _ => "<error>"
    }.to_string()
}


pub fn label_to_color_string(label: &str) -> String {
    match label {
        "Red" => "RED",
        "Orange" => "ORANGE",
        "Yellow" => "YELLOW",
        "Green" => "GREEN",
        "Blue" => "BLUE",
        "Purple" => "PERPLE", // note the typo in the game's code
        "White" => "WHITE",
        "Random" => "RANDOM",
        _ => "<error>",
    }.to_string()
}

pub const ENEMY_LIST: [(&str, &str); 96] = [
    ("ENEMY00", "Green Magmotamus"),
    ("ENEMY01", "Shelby"),
    ("ENEMY02", "Uniclod"),
    ("ENEMY03", "Buttonbee"),
    ("ENEMY04", "Slobba"),
    ("ENEMY05", "Sulkworm"),
    ("ENEMY06", "Dandan"),
    ("ENEMY07", "Jelly Jr."),
    ("ENEMY08", "Calderon"),
    ("ENEMY09", "Swadclod"),
    ("ENEMY10", "Sneak Sack"),
    ("ENEMY11", "Battins"),
    ("ENEMY12", "Candlemander"),
    ("ENEMY13", "Sea Jelly"),
    ("ENEMY14", "Whistle Soldier"),
    ("ENEMY15", "Sword Soldier"),
    ("ENEMY16", "Spear Soldier"),
    ("ENEMY17", "Cannon Soldier"),
    ("ENEMY18", "Large Sawgill"),
    ("ENEMY19", "Anemonee"),
    ("ENEMY20", "Danglerfish"),
    ("ENEMY21", "Bobber Clod"),
    ("ENEMY23", "Magmotamus"),
    ("ENEMY24", "Snip-Snap"),
    ("ENEMY25", "Gator"),
    ("ENEMY26", "Waddle Dee"),
    ("ENEMY27", "Spear Waddle Dee"),
    ("ENEMY28", "Waddle Doo"),
    ("ENEMY29", "Ooki"),
    ("ENEMY30", "Bomber"),
    ("ENEMY31", "Flamer"),
    ("ENEMY32", "Scarfy"),
    ("ENEMY33", "Blipper"),
    ("ENEMY34", "Buttonbug"),
    ("ENEMY35", "Bronto Burt"),
    ("ENEMY36", "Scared Soldier"),
    ("ENEMY37", "Grizzo"),
    ("ENEMY38", "Shotso"),
    ("ENEMY39", "Parasol Waddle Dee"),
    ("ENEMY40", "Chilly"),
    ("ENEMY41", "Waddle Dee (Duplicate)"),
    ("ENEMY42", "UFO"),
    ("ENEMY43", "Bow Waddle Dee"),
    ("ENEMY45", "Cyclod"),
    ("ENEMY46", "Buttonfly"),
    ("ENEMY48", "Space Jelly"),
    ("ENEMY49", "Truck Monster"),
    ("ENEMY50", "Large Cannon (Battleship Halberd)"),
    ("ENEMY51", "Small Cannon (Battleship Halberd)"),
    ("ENEMY52", "Battleship Halberd Turret"),
    ("ENEMY53", "Battleship Halberd Turret (Turret only)"),
    ("ENEMY54", "Battleship Halberd Flamethrower"),
    ("ENEMY55", "Podium (Cyclod)"),
    ("ENEMY56", "Battleship Halberd Flamethrower Barrier"),
    ("ENEMY57", "Orbitfly"),
    ("ENEMY58", "Spore Jelly"),
    ("ENEMY59", "UFO (Alt.)"),
    ("ENEMY60", "Dropso"),
    ("ENEMY61", "Stogue"),
    ("ENEMY62", "Capamari Tentacle"),
    ("ENEMY63", "Unidentified Enemy 63"),
    ("ENEMY64", "Unidentified Enemy 64"),
    ("ENEMY65", "Unidentified Enemy 65"),
    ("ENEMY66", "Unidentified Enemy 66"),
    ("ENEMY67", "Unidentified Enemy 67"),
    ("ENEMY68", "Unidentified Enemy 68"),
    ("ENEMY69", "Meta Knight's Sword"),
    ("ENEMY70", "Unidentified Enemy 70"),
    ("ENEMY71", "Unidentified Enemy 71"),
    ("ENEMY72", "Unidentified Enemy 72"),
    ("ENEMY74", "Unidentified Enemy 74"),
    ("ENEMY75", "Unidentified Enemy 75"),
    ("ENEMY76", "Smiley Face"),
    ("ENEMY78", "Unidentified Enemy 78"),
    ("ENEMY80", "Unidentified Enemy 80"),
    ("ENEMY81", "Unidentified Enemy 81"),
    ("ENEMY82", "Unidentified Enemy 82"),
    ("ENEMY83", "Unidentified Enemy 83"),
    ("ENEMY84", "Unidentified Enemy 84"),
    ("ENEMY100", "Small Sawgill"),
    ("ENEMY101", "Freezo"),
    ("ENEMY102", "Bobber Clod (Duplicate)"),
    ("ENEMY103", "Horizontal Battleship Halberd Barrier"),
    ("ENEMY106", "Lil' Kracko"),
    ("ENEMY107", "Kracko"),
    ("ENEMY110", "Emba"),
    ("ENEMY111", "Whistle Mariner"),
    ("ENEMY112", "Sword Mariner"),
    ("ENEMY113", "Spear Mariner"),
    ("ENEMY114", "Cannon Mariner"),
    ("ENEMY115", "Wicked Willow"),
    ("ENEMY116", "Cutfish"),
    ("ENEMY117", "Blast Mariner"),
    ("ENEMY118", "Scared Mariner"),
    ("ENEMY119", "Small Cannon (Moon Base)"),
    ("HELP_ROBOT", "Controls Screen"),
];

pub fn enemy_id_to_name(enemy_id: &str) -> &str {
    for (id, name) in ENEMY_LIST {
        if id == enemy_id {
            return name;
        }
    }

    return "<error>";
}
