use crate::quilt::common::*;
use byteorder::{ByteOrder, BigEndian};

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
    pub unk_10: Point2D,
    pub collision_type: String
}

#[derive(Default)]
pub struct LabeledWall {
    pub start: Point2D,
    pub end: Point2D,
    pub unk_10: Point2D,
    pub collision_type: String,
    pub label: String
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
    pub int_params_1: [i32; 3],
    pub float_params_1: [f32; 3],
    pub int_params_2: [i32; 4],
    pub float_params_2: [f32; 3],
    pub float_params_3: [f32; 3],
    pub string_params: [String; 3]
}

#[derive(Default)]
pub struct CommonGimmick {
    pub name: String,
    pub position: Point3D,
    pub params: CommonGimmickParams
}

#[derive(Default)]
pub struct Gimmick {
    pub name: String,
    pub unk_30: [u8; 0x10],
    pub position: Point3D,
    pub params: Params,

    pub is_selected: bool
}

#[derive(Default)]
pub struct Path {
    pub name: String,
    pub path_type: String,
    pub params: Params,
    pub points: Vec<Point2D>
}

#[derive(Default)]
pub struct Zone {
    pub name: String,
    pub unk_20: [u8; 0x20],
    pub params: Params,
    pub bounds_min: Point2D,
    pub bounds_max: Point2D
}


#[derive(Default)]
pub struct CourseInfo {
    pub name: String,
    pub unk_20: [u8; 0x20],
    pub params: Params,
    pub position: Point3D
}

#[derive(Default)]
pub struct Mapdata {
    pub unk_0: f32,
    pub bounds_min: Point2D,
    pub bounds_max: Point2D,

    pub walls: Vec<Wall>,
    pub labeled_walls: Vec<LabeledWall>,
    pub common_gimmicks: Vec<CommonGimmick>,
    pub gimmicks: Vec<Gimmick>,
    pub paths: Vec<Path>,
    pub zones: Vec<Zone>,
    pub course_infos: Vec<CourseInfo>,

    pub common_gimmick_names: NameMap,
    pub colbin_types: NameMap,
    pub wall_labels: NameMap,
}

impl Mapdata {
    pub fn from_data(input: &[u8]) -> Self {
        let mut mapdata = Mapdata::default();

        mapdata.unk_0 = BigEndian::read_f32(&input[..4]);
        mapdata.bounds_min.x = BigEndian::read_f32(&input[4..8]);
        mapdata.bounds_min.y = BigEndian::read_f32(&input[8..0xC]);
        mapdata.bounds_max.x = BigEndian::read_f32(&input[0xC..0x10]);
        mapdata.bounds_max.y = BigEndian::read_f32(&input[0x10..0x14]);

        // names and labels
        let num_common_gimmick_names_offs = BigEndian::read_u32(&input[0x4C..0x50]) as usize;
        let num_colbin_types_offs = BigEndian::read_u32(&input[0x50..0x54]) as usize;
        let num_wall_labels_offs = BigEndian::read_u32(&input[0x54..0x58]) as usize;

        let count = BigEndian::read_u32(
            &input[num_common_gimmick_names_offs..num_common_gimmick_names_offs + 4]
        ) as usize;
        
        mapdata.common_gimmick_names.read_names(
            input,
            count,
            0x20,
            
            4 + num_common_gimmick_names_offs
        );

        let count = BigEndian::read_u32(
            &input[num_colbin_types_offs..num_colbin_types_offs + 4]
        ) as usize;

        mapdata.colbin_types.read_names(
            input,
            count,
            0x20,
            4 + num_colbin_types_offs
        );

        let count = BigEndian::read_u32(
            &input[num_wall_labels_offs..num_wall_labels_offs + 4]
        ) as usize;

        mapdata.wall_labels.read_names(
            input,
            count,
            0x20,
            4 + num_wall_labels_offs
        );

        // entities

        // walls

        let num_walls = BigEndian::read_u32(&input[0x14..0x18]) as usize;
        let wall_offs = BigEndian::read_u32(&input[0x18..0x1C]) as usize;

        for i in 0..num_walls {
            let start = wall_offs + (i * WALL_SIZE);
            let end = start + WALL_SIZE;
            
            mapdata.walls.push(Wall::from_bytes(
                &input[start..end],
                &mapdata.colbin_types
            ));
        }
        
        // labeled walls
        
        let num_labeled_walls = BigEndian::read_u32(&input[0x1C..0x20]) as usize;
        let labeled_wall_offs = BigEndian::read_u32(&input[0x20..0x24]) as usize;
        
        for i in 0..num_labeled_walls {
            let start = labeled_wall_offs + (i * LABELED_WALL_SIZE);
            let end = start + LABELED_WALL_SIZE;

            mapdata.labeled_walls.push(LabeledWall::from_bytes(
                &input[start..end],
                &mapdata.colbin_types,
                &mapdata.wall_labels
            ));
        }

        // common gimmicks

        let num_common_gimmicks = BigEndian::read_u32(&input[0x24..0x28]) as usize;
        let common_gimmick_offs = BigEndian::read_u32(&input[0x28..0x2C]) as usize;

        for i in 0..num_common_gimmicks {
            let start = common_gimmick_offs + (i * COMMON_GIMMICK_SIZE);
            let end = start + COMMON_GIMMICK_SIZE;


            mapdata.common_gimmicks.push(CommonGimmick::from_bytes(
                &input[start..end],
                &mapdata.common_gimmick_names
            ));

        }

        // gimmicks

        let num_gimmicks = BigEndian::read_u32(&input[0x2C..0x30]) as usize;
        let gimmick_offs = BigEndian::read_u32(&input[0x30..0x34]) as usize;

        for i in 0..num_gimmicks {
            let start = gimmick_offs + (i * GIMMICK_SIZE);
            let end = start + GIMMICK_SIZE;

            mapdata.gimmicks.push(Gimmick::from_bytes(
                &input[start..end]
            ));
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

            mapdata.paths.push(Path::from_bytes(
                &input[start..end]
            ));
        }

        // zones

        let num_zones = BigEndian::read_u32(&input[0x3C..0x40]) as usize;
        let zone_offs = BigEndian::read_u32(&input[0x40..0x44]) as usize;

        for i in 0..num_zones {
            let start = zone_offs + (i * ZONE_SIZE);
            let end = start + ZONE_SIZE;

            mapdata.zones.push(Zone::from_bytes(
                &input[start..end]
            ));
        }

        // course info

        let num_course_info = BigEndian::read_u32(&input[0x44..0x48]) as usize;
        let course_info_offs = BigEndian::read_u32(&input[0x48..0x4C]) as usize;

        for i in 0..num_course_info {
            let start = course_info_offs + (i * COURSE_INFO_SIZE);
            let end = start + COURSE_INFO_SIZE;

            mapdata.course_infos.push(CourseInfo::from_bytes(
                &input[start..end]
            ));
        }

        mapdata
    }
}


impl Wall {
    fn from_bytes(input: &[u8], name_map: &NameMap) -> Self {
        let mut wall = Self::default();

        wall.start = Point2D::from_be_bytes(&input[..8]);
        wall.end = Point2D::from_be_bytes(&input[8..0x10]);
        wall.unk_10 = Point2D::from_be_bytes(&input[0x10..0x18]);
        
        let type_index = BigEndian::read_u32(&input[0x1C..0x20]) as usize;

        wall.collision_type = name_map.names[type_index].clone();

        wall
    }
}

impl LabeledWall {
    fn from_bytes(input: &[u8], collision_type_map: &NameMap, label_map: &NameMap) -> Self {
        let mut wall = Self::default();

        wall.start = Point2D::from_be_bytes(&input[..8]);
        wall.end = Point2D::from_be_bytes(&input[8..0x10]);
        wall.unk_10 = Point2D::from_be_bytes(&input[0x10..0x18]);
        
        let type_index = BigEndian::read_u32(&input[0x1C..0x20]) as usize;

        wall.collision_type = collision_type_map.names[type_index].clone();

        let label_index = BigEndian::read_u32(&input[0x20..0x24]) as usize;

        wall.label = label_map.names[label_index].clone();

        wall
    }
}

impl Params {
    fn from_bytes(input: &[u8]) -> Self {
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
}

impl CommonGimmickParams {
    fn from_bytes(input: &[u8]) -> Self {
        let mut params = Self::default();

        for i in 0..3 {
            let start = i * 4;
            let end = start + 4;
            params.int_params_1[i] = BigEndian::read_i32(&input[start..end]);
        }
        
        for i in 0..3 {
            let start = 0xC + (i * 4);
            let end = start + 4;
            params.float_params_1[i] = BigEndian::read_f32(&input[start..end]);
        }
        
        for i in 0..4 {
            let start = 0x18 + (i * 4);
            let end = start + 4;
            params.int_params_2[i] = BigEndian::read_i32(&input[start..end]);
        }

        for i in 0..3 {
            let start = 0x28 + (i * 4);
            let end = start + 4;
            params.float_params_2[i] = BigEndian::read_f32(&input[start..end]);
        }

        for i in 0..3 {
            let start = 0x34 + (i * 4);
            let end = start + 4;
            params.float_params_3[i] = BigEndian::read_f32(&input[start..end]);
        }

        for i in 0..3 {
            let start = 0x40 + (i * 64);
            let end = start + 64;
            params.string_params[i] = string_from_buffer(&input[start..end]);
        }

        params
    }
}

impl CommonGimmick {
    fn from_bytes(input: &[u8], name_map: &NameMap) -> Self {
        let mut gmk = Self::default();

        let name_index = BigEndian::read_u32(&input[0..4]) as usize;
        gmk.name = name_map.names[name_index].clone();
        gmk.position = Point3D::from_be_bytes(&input[4..0x10]);
        gmk.params = CommonGimmickParams::from_bytes(&input[0x10..]);

        gmk
    }
}

impl Gimmick {
    fn from_bytes(input: &[u8]) -> Self {
        let mut gmk = Self::default();

        gmk.name = string_from_buffer(&input[..0x30]);

        gmk.unk_30.copy_from_slice(&input[0x30..0x40]);
        gmk.position = Point3D::from_be_bytes(&input[0x40..0x4C]);
        gmk.params = Params::from_bytes(&input[0x4C..]);
        

        gmk
    }
}

impl Path {
    fn from_bytes(input: &[u8]) -> Self {
        let mut path = Self::default();

        path.name = string_from_buffer(&input[..0x20]);
        path.path_type = string_from_buffer(&input[0x20..0x40]);
        path.params = Params::from_bytes(&input[0x40..0x118]);

        let num_points = BigEndian::read_u32(&input[0x118..0x11C]) as usize;

        for i in 0..num_points {
            let start = 0x11C + (i * 8);
            let end = start + 8;
            path.points.push(Point2D::from_be_bytes(
                &input[start..end]
            ));
        }


        path
    }
}


impl Zone {
    fn from_bytes(input: &[u8]) -> Self {
        let mut zone = Self::default();

        zone.name = string_from_buffer(&input[..0x20]);
        zone.unk_20.copy_from_slice(&input[0x20..0x40]);
        zone.params = Params::from_bytes(&input[0x40..0x118]);
        zone.bounds_min = Point2D::from_be_bytes(&input[0x118..0x120]);
        zone.bounds_max = Point2D::from_be_bytes(&input[0x120..0x128]);

        zone
    }
}

impl CourseInfo {
    fn from_bytes(input: &[u8]) -> Self {
        let mut info = Self::default();

        info.name = string_from_buffer(&input[..0x20]);
        info.unk_20.copy_from_slice(&input[0x20..0x40]);
        info.params = Params::from_bytes(&input[0x40..0x118]);
        info.position = Point3D::from_be_bytes(&input[0x118..]);
        
        info
    }
}
