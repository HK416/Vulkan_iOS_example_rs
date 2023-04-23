use crate::math::*;



pub const MAX_OBJECTS_NUM: usize = 5_000;


pub const TRIANGLE_POSITIONS: [Vec3; 3] = [
    Vec3::new_vector(-0.5, -0.25, 0.0),
    Vec3::new_vector(0.5, -0.25, 0.0),
    Vec3::new_vector(0.0, 0.5, 0.0),
];


pub const QUAD_INDICES: [u16; 6] = [ 
    0, 1, 2, 
    2, 3, 0 
];
pub const QUAD_POSITIONS: [Vec3; 4] = [
    Vec3::new_vector(-1.0, 1.0, 0.0),
    Vec3::new_vector(-1.0, -1.0, 0.0),
    Vec3::new_vector(1.0, -1.0, 0.0),
    Vec3::new_vector(1.0, 1.0, 0.0),
];


pub const CUBE_INDICES: [u16; 36] = [
    3, 2, 0, 0, 1, 3, // top
    2, 6, 4, 4, 0, 2, // front
    0, 4, 5, 5, 1, 0, // right
    3, 2, 6, 6, 7, 3, // left
    5, 1, 3, 3, 7, 5, // back
    6, 4, 5, 5, 7, 6, // bottom
];
pub const CUBE_POSITIONS: [Vec3; 8] = [
    Vec3::new_vector(1.0, 1.0, 1.0), // 0
    Vec3::new_vector(1.0, 1.0, -1.0), // 1
    Vec3::new_vector(-1.0, 1.0, 1.0), // 2
    Vec3::new_vector(-1.0, 1.0, -1.0), // 3
    Vec3::new_vector(1.0, -1.0, 1.0), // 4
    Vec3::new_vector(1.0, -1.0, -1.0), // 5
    Vec3::new_vector(-1.0, -1.0, 1.0), // 6
    Vec3::new_vector(-1.0, -1.0, -1.0) // 7
];


pub const VERT_SHADER_PATH: &'static str = "shaders/vert.spv";
pub const FRAG_SHADER_PATH: &'static str = "shaders/frag.spv";
