use crate::quilt::common::*;
use byteorder::{BigEndian, ByteOrder};

const HEADER_SIZE: usize = 0x58;
const WALL_SIZE: usize = 0x20;
const LABELED_WALL_SIZE: usize = 0x24;
const _PARAMS_SIZE: usize = 0xD8;
const _COMMON_GIMMICK_PARAMS_SIZE: usize = 0x100;
const COMMON_GIMMICK_SIZE: usize = 400;
const GIMMICK_SIZE: usize = 0x124;
const ZONE_SIZE: usize = 296;
const COURSE_INFO_SIZE: usize = 292;
const BASE_PATH_SIZE: usize = 0x11C;

#[derive(Default)]
pub struct Wall {
    pub start: Point2D,
    pub end: Point2D,
    pub normalized_vector: Point2D, // this field is a bit odd in that x and y are swapped
    pub collision_type: String,

    pub is_selected: bool,
}

#[derive(Default)]
pub struct LabeledWall {
    pub start: Point2D,
    pub end: Point2D,
    pub normalized_vector: Point2D, // this is an angle of some sort
    pub collision_type: String,
    pub label: String,

    pub is_selected: bool,
}

#[derive(Default)]
pub struct Params {
    pub int_params: [i32; 3],
    pub float_params: [f32; 3],
    pub string_params: [String; 3],
}

#[derive(Default)]
/// Many fields are unknown.
pub struct CommonGimmickParams {
    pub common_int_params: [i32; 2],
    pub common_float_params: [f32; 2],
    pub common_string_param: String,
    pub int_params: [i32; 5],
    pub float_params: [f32; 5],
    pub string_params: [String; 5],
}

#[derive(Default)]
pub struct CommonGimmick {
    pub hex: String,
    pub position: Point3D,
    pub params: CommonGimmickParams,

    pub is_selected: bool,
}

#[derive(Default)]
pub struct Gimmick {
    pub name: String,
    pub unk_30: [u8; 0x10],
    pub position: Point3D,
    pub params: Params,

    pub is_selected: bool,
}

#[derive(Default)]
pub struct Path {
    pub name: String,
    pub path_type: String,
    pub params: Params,
    pub points: Vec<Point2D>,

    pub is_selected: bool,
}

#[derive(Default)]
pub struct Zone {
    pub name: String,
    pub unk_20: String,
    pub params: Params,
    pub bounds_start: Point2D,
    pub bounds_end: Point2D,

    pub is_selected: bool,
}

#[derive(Default)]
pub struct CourseInfo {
    pub name: String,
    pub unk_20: String,
    pub params: Params,
    pub position: Point3D,

    pub is_selected: bool,
}

#[derive(Default)]
pub struct Mapdata {
    pub _version: f32,
    pub bounds_min: Point2D,
    pub bounds_max: Point2D,

    pub walls: Vec<Wall>,
    pub labeled_walls: Vec<LabeledWall>,
    pub common_gimmicks: Vec<CommonGimmick>,
    pub gimmicks: Vec<Gimmick>,
    pub paths: Vec<Path>,
    pub zones: Vec<Zone>,
    pub course_infos: Vec<CourseInfo>,

    pub common_gimmick_names: HexMap,
    pub colbin_types: NameMap,
    pub wall_labels: NameMap,
}

impl Mapdata {
    pub fn decode(input: &[u8]) -> Self {
        let version = BigEndian::read_f32(&input[..4]);

        let bounds_min = Point2D::from_be_bytes(&input[4..0xC]);
        let bounds_max = Point2D::from_be_bytes(&input[0xC..0x14]);

        let mut mapdata = Mapdata {
            _version: version,
            bounds_min,
            bounds_max,
            ..Default::default()
        };

        // names and labels
        let num_common_gimmick_names_offs = BigEndian::read_u32(&input[0x4C..0x50]) as usize;
        let num_colbin_types_offs = BigEndian::read_u32(&input[0x50..0x54]) as usize;
        let num_wall_labels_offs = BigEndian::read_u32(&input[0x54..0x58]) as usize;

        let count = BigEndian::read_u32(
            &input[num_common_gimmick_names_offs..num_common_gimmick_names_offs + 4],
        ) as usize;

        mapdata.common_gimmick_names.read_names(
            input,
            count,
            0x20,
            4 + num_common_gimmick_names_offs,
        );

        let count =
            BigEndian::read_u32(&input[num_colbin_types_offs..num_colbin_types_offs + 4]) as usize;

        mapdata
            .colbin_types
            .read_names(input, count, 0x20, 4 + num_colbin_types_offs);

        let count =
            BigEndian::read_u32(&input[num_wall_labels_offs..num_wall_labels_offs + 4]) as usize;

        mapdata
            .wall_labels
            .read_names(input, count, 0x20, 4 + num_wall_labels_offs);

        // entities

        // walls

        let num_walls = BigEndian::read_u32(&input[0x14..0x18]) as usize;
        let wall_offs = BigEndian::read_u32(&input[0x18..0x1C]) as usize;

        for i in 0..num_walls {
            let start = wall_offs + (i * WALL_SIZE);
            let end = start + WALL_SIZE;

            mapdata
                .walls
                .push(Wall::decode(&input[start..end], &mapdata.colbin_types));
        }

        // labeled walls

        let num_labeled_walls = BigEndian::read_u32(&input[0x1C..0x20]) as usize;
        let labeled_wall_offs = BigEndian::read_u32(&input[0x20..0x24]) as usize;

        for i in 0..num_labeled_walls {
            let start = labeled_wall_offs + (i * LABELED_WALL_SIZE);
            let end = start + LABELED_WALL_SIZE;

            mapdata.labeled_walls.push(LabeledWall::decode(
                &input[start..end],
                &mapdata.colbin_types,
                &mapdata.wall_labels,
            ));
        }

        // common gimmicks

        let num_common_gimmicks = BigEndian::read_u32(&input[0x24..0x28]) as usize;
        let common_gimmick_offs = BigEndian::read_u32(&input[0x28..0x2C]) as usize;

        for i in 0..num_common_gimmicks {
            let start = common_gimmick_offs + (i * COMMON_GIMMICK_SIZE);
            let end = start + COMMON_GIMMICK_SIZE;

            mapdata.common_gimmicks.push(CommonGimmick::decode(
                &input[start..end],
                &mapdata.common_gimmick_names,
            ));
        }

        // gimmicks

        let num_gimmicks = BigEndian::read_u32(&input[0x2C..0x30]) as usize;
        let gimmick_offs = BigEndian::read_u32(&input[0x30..0x34]) as usize;

        for i in 0..num_gimmicks {
            let start = gimmick_offs + (i * GIMMICK_SIZE);
            let end = start + GIMMICK_SIZE;

            mapdata
                .gimmicks
                .push(Gimmick::decode(&input[start..end]));
        }

        // paths

        let num_paths = BigEndian::read_u32(&input[0x34..0x38]) as usize;
        let path_offs = BigEndian::read_u32(&input[0x38..0x3C]) as usize;

        let mut cur_path_offs = 0; // necessary because paths are variable-length
        for i in 0..num_paths {
            let start = cur_path_offs + path_offs + (i * BASE_PATH_SIZE);
            let num_points = BigEndian::read_u32(&input[start + 0x118..start + 0x11C]) as usize;
            let end = start + BASE_PATH_SIZE + (num_points * 8);
            cur_path_offs += num_points * 8;

            mapdata.paths.push(Path::decode(&input[start..end]));
        }

        // zones

        let num_zones = BigEndian::read_u32(&input[0x3C..0x40]) as usize;
        let zone_offs = BigEndian::read_u32(&input[0x40..0x44]) as usize;

        for i in 0..num_zones {
            let start = zone_offs + (i * ZONE_SIZE);
            let end = start + ZONE_SIZE;

            mapdata.zones.push(Zone::decode(&input[start..end]));
        }

        // course info

        let num_course_info = BigEndian::read_u32(&input[0x44..0x48]) as usize;
        let course_info_offs = BigEndian::read_u32(&input[0x48..0x4C]) as usize;

        for i in 0..num_course_info {
            let start = course_info_offs + (i * COURSE_INFO_SIZE);
            let end = start + COURSE_INFO_SIZE;

            mapdata
                .course_infos
                .push(CourseInfo::decode(&input[start..end]));
        }

        mapdata
    }

    pub fn encode(&mut self) -> Vec<u8> {
        // preparations

        // determine if any names haven't been added to the maps
        for wall in self.walls.iter() {
            if !self.colbin_types.names.contains(&wall.collision_type) {
                self.colbin_types.names.push(wall.collision_type.clone());
            }
        }

        for wall in self.labeled_walls.iter() {
            if !self.colbin_types.names.contains(&wall.collision_type) {
                self.colbin_types.names.push(wall.collision_type.clone());
            }

            if !self.wall_labels.names.contains(&wall.label) {
                self.wall_labels.names.push(wall.label.clone());
            }
        }

        // calculate offsets
        let wall_offset = HEADER_SIZE;
        let labeled_wall_offset = wall_offset + (WALL_SIZE * self.walls.len());
        let common_gimmick_offset =
            labeled_wall_offset + (LABELED_WALL_SIZE * self.labeled_walls.len());
        let gimmick_offset =
            common_gimmick_offset + (COMMON_GIMMICK_SIZE * self.common_gimmicks.len());
        let path_offset = gimmick_offset + (GIMMICK_SIZE * self.gimmicks.len());

        // paths have a variable length
        let mut path_chunk_size = 0;
        for path in self.paths.iter() {
            path_chunk_size += BASE_PATH_SIZE + (8 * path.points.len());
        }

        let zone_offset = path_offset + path_chunk_size;
        let course_info_offset = zone_offset + (ZONE_SIZE * self.zones.len());
        let common_gimmick_name_offset =
            course_info_offset + (COURSE_INFO_SIZE * self.course_infos.len());

        let colbin_type_offset =
            common_gimmick_name_offset + 4 + (0x20 * self.common_gimmick_names.hex_names.len());
        let labeled_wall_labels_offset =
            colbin_type_offset + 4 + (0x20 * self.colbin_types.names.len());

        // data writing
        let mut out = Vec::<u8>::new();

        // header
        let version: f32 = 3.3; // hardcode because there's no reason to use any other version

        out.extend(version.to_be_bytes());
        out.extend(self.bounds_min.get_be_bytes());
        out.extend(self.bounds_max.get_be_bytes());
        out.extend((self.walls.len() as u32).to_be_bytes());
        out.extend((wall_offset as u32).to_be_bytes());
        out.extend((self.labeled_walls.len() as u32).to_be_bytes());
        out.extend((labeled_wall_offset as u32).to_be_bytes());
        out.extend((self.common_gimmicks.len() as u32).to_be_bytes());
        out.extend((common_gimmick_offset as u32).to_be_bytes());
        out.extend((self.gimmicks.len() as u32).to_be_bytes());
        out.extend((gimmick_offset as u32).to_be_bytes());
        out.extend((self.paths.len() as u32).to_be_bytes());
        out.extend((path_offset as u32).to_be_bytes());
        out.extend((self.zones.len() as u32).to_be_bytes());
        out.extend((zone_offset as u32).to_be_bytes());
        out.extend((self.course_infos.len() as u32).to_be_bytes());
        out.extend((course_info_offset as u32).to_be_bytes());
        out.extend((common_gimmick_name_offset as u32).to_be_bytes());
        out.extend((colbin_type_offset as u32).to_be_bytes());
        out.extend((labeled_wall_labels_offset as u32).to_be_bytes());

        // walls
        for (i, w) in self.walls.iter().enumerate() {
            out.extend(w.encode(i, &self.colbin_types));
        }

        // labeled walls
        for (i, w) in self.labeled_walls.iter().enumerate() {
            out.extend(w.encode(i, &self.colbin_types, &self.wall_labels));
        }

        // common gimmicks
        for gmk in self.common_gimmicks.iter() {
            out.extend(gmk.encode(&self.common_gimmick_names));
        }

        // gimmicks
        for gmk in self.gimmicks.iter() {
            out.extend(gmk.encode());
        }

        // paths
        for path in self.paths.iter() {
            out.extend(path.encode());
        }

        // zones
        for zone in self.zones.iter() {
            out.extend(zone.encode());
        }

        // course info
        for info in self.course_infos.iter() {
            out.extend(info.encode());
        }

        // common gimmick names
        out.extend((self.common_gimmick_names.hex_names.len() as u32).to_be_bytes());
        for name in self.common_gimmick_names.hex_names.iter() {
            let bytes = hex::decode(name).unwrap();
            let mut padded_bytes = bytes;

            if padded_bytes.len() < 0x20 {
                padded_bytes.resize(0x20, 0);
            }
            out.extend(padded_bytes);
        }

        // colbin collision types
        out.extend((self.colbin_types.names.len() as u32).to_be_bytes());
        for col_type in self.colbin_types.names.iter() {
            out.extend(string_to_buffer(col_type, 0x20));
        }

        // labeled wall labels
        out.extend((self.wall_labels.names.len() as u32).to_be_bytes());
        for label in self.wall_labels.names.iter() {
            out.extend(string_to_buffer(label, 0x20));
        }

        // alignment
        out.resize(out.len().next_multiple_of(0x20), 0);

        out
    }
}

impl Wall {
    fn decode(input: &[u8], name_map: &NameMap) -> Self {
        let start = Point2D::from_be_bytes(&input[..8]);
        let end = Point2D::from_be_bytes(&input[8..0x10]);
        let normalized_vector = Point2D::from_be_bytes(&input[0x10..0x18]);
        let type_index = BigEndian::read_u32(&input[0x1C..0x20]) as usize;
        let collision_type = name_map.names[type_index].clone();

        Wall {
            start,
            end,
            normalized_vector,
            collision_type,
            ..Default::default()
        }
    }

    pub fn encode(&self, wall_index: usize, name_map: &NameMap) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend(self.start.get_be_bytes());
        out.extend(self.end.get_be_bytes());
        out.extend(self.normalized_vector.get_be_bytes());

        out.extend((wall_index as u32).to_be_bytes());

        let type_index = name_map
            .names
            .iter()
            .position(|name| name == &self.collision_type)
            .expect("collision_type not found in name_map");

        out.extend((type_index as u32).to_be_bytes());

        out
    }

    pub fn set_normalized_vector(&mut self) {
        let direction = (self.end.x - self.start.x, self.end.y - self.start.y);
        let magnitude = f32::sqrt(direction.0.powf(2.0) + direction.1.powf(2.0));
        let normalized = (direction.0 / magnitude, direction.1 / magnitude);

        self.normalized_vector = Point2D {
            x: -normalized.1, // this has to be inverted,
            // because otherwise, the player's NURBS animation
            // (namely, the player's feet) would face the opposite direction
            y: normalized.0,
        };
    }
}

impl LabeledWall {
    fn decode(input: &[u8], collision_type_map: &NameMap, label_map: &NameMap) -> Self {
        let start = Point2D::from_be_bytes(&input[..8]);
        let end = Point2D::from_be_bytes(&input[8..0x10]);
        let normalized_vector = Point2D::from_be_bytes(&input[0x10..0x18]);
        let type_index = BigEndian::read_u32(&input[0x1C..0x20]) as usize;
        let collision_type = collision_type_map.names[type_index].clone();
        let label_index = BigEndian::read_u32(&input[0x20..0x24]) as usize;
        let label = label_map.names[label_index].clone();

        LabeledWall {
            start,
            end,
            normalized_vector,
            collision_type,
            label,
            ..Default::default()
        }
    }

    pub fn encode(
        &self,
        index: usize,
        collision_type_map: &NameMap,
        label_map: &NameMap,
    ) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend(self.start.get_be_bytes());

        out.extend(self.end.get_be_bytes());

        out.extend(self.normalized_vector.get_be_bytes());

        out.extend((index as u32).to_be_bytes());

        let type_index = collision_type_map
            .names
            .iter()
            .position(|name| name == &self.collision_type)
            .expect("collision_type not found in collision_type_map");
        out.extend((type_index as u32).to_be_bytes());

        let label_index = label_map
            .names
            .iter()
            .position(|name| name == &self.label)
            .expect("label not found in label_map");
        out.extend((label_index as u32).to_be_bytes());

        out
    }

    pub fn set_normalized_vector(&mut self) {
        let direction = (self.end.x - self.start.x, self.end.y - self.start.y);
        let magnitude = f32::sqrt(direction.0.powf(2.0) + direction.1.powf(2.0));
        let normalized = (direction.0 / magnitude, direction.1 / magnitude);

        self.normalized_vector = Point2D {
            x: -normalized.1, // this has to be inverted,
            // because otherwise, the player's NURBS animation
            // (namely, the player's feet) would face the opposite direction
            y: normalized.0,
        };
    }
}

impl Params {
    fn decode(input: &[u8]) -> Self {
        let mut params = Self::default();

        for i in 0..3 {
            let start = i * 4;
            let end = start + 4;
            params.int_params[i] = BigEndian::read_i32(&input[start..end]);
        }

        for i in 0..3 {
            let start = 0xC + (i * 4);
            let end = start + 4;
            params.float_params[i] = BigEndian::read_f32(&input[start..end]);
        }

        for i in 0..3 {
            let start = 0x18 + (i * 64);
            let end = start + 64;

            params.string_params[i] = string_from_buffer(&input[start..end]);
        }

        params
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        for int in &self.int_params {
            out.extend(int.to_be_bytes());
        }

        for float in &self.float_params {
            out.extend(float.to_be_bytes());
        }

        for string in &self.string_params {
            let mut buffer = [0u8; 64];
            let bytes = string.as_bytes();
            let len = bytes.len().min(64); // truncate if necessary
            buffer[..len].copy_from_slice(&bytes[..len]);
            out.extend(&buffer);
        }

        out
    }
}

impl CommonGimmickParams {
    fn decode(input: &[u8]) -> Self {
        let mut params = Self::default();

        for i in 0..2 {
            let start = i * 4;
            let end = start + 4;
            params.common_int_params[i] = BigEndian::read_i32(&input[start..end]);
        }

        for i in 0..2 {
            let start = 8 + (i * 4);
            let end = start + 4;
            params.common_float_params[i] = BigEndian::read_f32(&input[start..end]);
        }

        params.common_string_param = string_from_buffer(&input[0x10..0x18]);

        for i in 0..5 {
            let start = 0x18 + (i * 4);
            let end = start + 4;
            params.int_params[i] = BigEndian::read_i32(&input[start..end]);
        }

        for i in 0..5 {
            let start = 0x2C + (i * 4);
            let end = start + 4;
            params.float_params[i] = BigEndian::read_f32(&input[start..end]);
        }

        for i in 0..5 {
            let start = 0x40 + (i * 64);
            let end = start + 64;
            params.string_params[i] = string_from_buffer(&input[start..end]);
        }

        params
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        for int in &self.common_int_params {
            out.extend(int.to_be_bytes());
        }

        for float in &self.common_float_params {
            out.extend(float.to_be_bytes());
        }

        let mut common_string_buffer = [0u8; 8];
        let bytes = self.common_string_param.as_bytes();
        let len = bytes.len().min(8); // truncate if necessary
        common_string_buffer[..len].copy_from_slice(&bytes[..len]);
        out.extend(&common_string_buffer);

        for int in &self.int_params {
            out.extend(int.to_be_bytes());
        }

        for float in &self.float_params {
            out.extend(float.to_be_bytes());
        }

        for string in &self.string_params {
            let mut string_buffer = [0u8; 64];
            let bytes = string.as_bytes();
            let len = bytes.len().min(64); // truncate if necessary
            string_buffer[..len].copy_from_slice(&bytes[..len]);
            out.extend(&string_buffer);
        }

        out
    }
}

impl CommonGimmick {
    fn decode(input: &[u8], name_map: &HexMap) -> Self {
        let mut gmk = Self::default();

        let name_index = BigEndian::read_u32(&input[0..4]) as usize;
        gmk.hex = name_map.hex_names[name_index].clone();
        gmk.position = Point3D::from_be_bytes(&input[4..0x10]);
        gmk.params = CommonGimmickParams::decode(&input[0x10..]);

        gmk
    }

    pub fn encode(&self, name_map: &HexMap) -> Vec<u8> {
        let mut out = Vec::new();

        // name index
        if let Some(index) = name_map.hex_names.iter().position(|name| name == &self.hex) {
            out.extend((index as u32).to_be_bytes());
        } else {
            panic!("Hex string '{}' not found in name_map", self.hex);
        }

        // position
        out.extend(self.position.to_be_bytes());

        // params
        out.extend(self.params.encode());

        out
    }
}

impl Gimmick {
    fn decode(input: &[u8]) -> Self {
        let name = string_from_buffer(&input[..0x30]);
        let position = Point3D::from_be_bytes(&input[0x40..0x4C]);
        let params = Params::decode(&input[0x4C..]);

        let mut gmk = Gimmick {
            name,
            position,
            params,
            ..Default::default()
        };

        gmk.unk_30.copy_from_slice(&input[0x30..0x40]);

        gmk
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend(self.name.as_bytes());
        out.extend(vec![0; 0x30 - self.name.len()]);

        out.extend_from_slice(&self.unk_30);

        out.extend(self.position.to_be_bytes());

        out.extend(self.params.encode());

        out
    }
}

impl Path {
    fn decode(input: &[u8]) -> Self {
        let name = string_from_buffer(&input[..0x20]);
        let path_type = string_from_buffer(&input[0x20..0x40]);
        let params = Params::decode(&input[0x40..0x118]);

        let mut path = Path {
            name,
            path_type,
            params,
            ..Default::default()
        };

        let num_points = BigEndian::read_u32(&input[0x118..0x11C]) as usize;

        for i in 0..num_points {
            let start = 0x11C + (i * 8);
            let end = start + 8;
            path.points.push(Point2D::from_be_bytes(&input[start..end]));
        }

        path
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend(self.name.as_bytes());
        out.extend(vec![0; 0x20 - self.name.len()]);

        out.extend(self.path_type.as_bytes());
        out.extend(vec![0; 0x20 - self.path_type.len()]);

        out.extend(self.params.encode());

        out.extend((self.points.len() as u32).to_be_bytes());

        for point in &self.points {
            out.extend(point.get_be_bytes());
        }

        out
    }
}

impl Zone {
    fn decode(input: &[u8]) -> Self {
        let name = string_from_buffer(&input[..0x20]);
        let unk_20 = string_from_buffer(&input[0x20..0x40]);
        let params = Params::decode(&input[0x40..0x118]);
        let bounds_start = Point2D::from_be_bytes(&input[0x118..0x120]);
        let bounds_end = Point2D::from_be_bytes(&input[0x120..0x128]);

        Zone {
            name,
            unk_20,
            params,
            bounds_start,
            bounds_end,
            ..Default::default()
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend(self.name.as_bytes());
        out.extend(vec![0; 0x20 - self.name.len()]);

        out.extend(self.unk_20.as_bytes());
        out.extend(vec![0; 0x20 - self.unk_20.len()]);

        out.extend(self.params.encode());

        out.extend(self.bounds_start.get_be_bytes());

        out.extend(self.bounds_end.get_be_bytes());

        out
    }
}

impl CourseInfo {
    fn decode(input: &[u8]) -> Self {
        let name = string_from_buffer(&input[..0x20]);
        let unk_20 = string_from_buffer(&input[0x20..0x40]);
        let params = Params::decode(&input[0x40..0x118]);
        let position = Point3D::from_be_bytes(&input[0x118..]);

        CourseInfo {
            name,
            unk_20,
            params,
            position,
            ..Default::default()
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend(self.name.as_bytes());
        out.extend(vec![0; 0x20 - self.name.len()]);

        out.extend(self.unk_20.as_bytes());
        out.extend(vec![0; 0x20 - self.unk_20.len()]);

        out.extend(self.params.encode());

        out.extend(self.position.to_be_bytes());

        out
    }
}

// other types may exist, but they're either
// unused or get coerced into other types
// (e.g. "NML_S" -> "NML_SOFT").
// this list is all that was found in the game's binary
pub const COLLISION_TYPES: [&str; 60] = [
    "NML",
    "CANCEL_METAMO",
    "GO_HEAVEN",
    "GO_SEC",
    "THROUGH",
    "SLOW",
    "NONE_SLIP",
    "NML_URA",
    "THROUGH_URA",
    "NML_SOFT",
    "THROUGH_SOFT",
    "NML_HARD",
    "THROUGH_HARD",
    "CAMERA",
    "CAMERA_THROUGH",
    "CAMERA_PLAYER",
    "CAMERA_PLAYER_Y",
    "CAMERA_MORI",
    "CAMERA_NML_PLAYER",
    "CAMERA_M",
    "CAMERA_THROUGH_M",
    "CAMERA_PLAYER_M",
    "CAMERA_PLAYER_Y_M",
    "CAMERA_NML_PLAYER_M",
    "DAMAGE",
    "ONE_DEAD",
    "IGNORE_PLAYER",
    "QUICKSAND",
    "ENT_TUNNEL",
    "CART",
    "DESTROY_GIMMICK",
    "NML_SLIP",
    "THROUGH_SLIP",
    "SPIN",
    "SPIN_DMG",
    "SPIN_CORRECT",
    "NML_BEAD_NS",
    "NML_BEAD_NS_HARD",
    "NML_BEAD_NS_URA",
    "THROUGH_BEAD_NS",
    "BEAD_ONLY",
    "NML_SIT_FLOOR",
    "THROUGH_SIT_FLOOR",
    "DMG_FIRE",
    "DMG_ICE",
    "DMG_THUNDER",
    "DELETE_ENEMY",
    "ACCEL_GROUND",
    "PLAYER_ONLY",
    "PLAYER_ONLY_THROUGH",
    "PL_PENDULUM_REFLECT",
    "RSTONE_COMMAND",
    "REFLECT_CAPTURE_OBJ",
    "NML_ICE",
    "THROUGH_ICE",
    "GO_HEAVEN_FIRE",
    "GO_HEAVEN_FIRE_PL_ONLY",
    "PULL_ENEMY",
    "NONE",
    "THROUGH_TRAIN_LIMIT",
];
