mod vec2;
mod vec3;
mod vec4;
mod quat;

mod mat2;
mod mat3;
mod mat4;

pub use vec2::*;
pub use vec3::*;
pub use vec4::*;
pub use quat::*;

pub use mat2::*;
pub use mat3::*;
pub use mat4::*;

/// return the orthographic projection matrix. (left-handed coordinate system)
/// 
/// ## Panics
/// if `view_width` and `view_height` are less than or equal to 0, 
/// if `near_z` is less than or equal to 0, 
/// and if `far_z` is less than or equal to `near_z`, 
/// program execution is stopped.
/// 
#[inline]
pub fn orthographic_lh(
    view_width: f32,
    view_height: f32,
    near_z: f32,
    far_z: f32
) -> Mat4x4 {
    assert!(0.0 < view_width, "view width must be greater than zero.");
    assert!(0.0 < view_height, "view height must be greater than zero.");
    assert!(0.0 <= near_z && near_z < far_z, "far-z must be greater than near-z.");

    Mat4x4 {
        r1c1: 2.0 / view_width,
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 2.0 / view_height,
        r2c3: 0.0,
        r2c4: 0.0,

        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: 1.0 / (far_z - near_z),
        r3c4: 0.0,

        r4c1: 0.0,
        r4c2: 0.0,
        r4c3: -near_z / (far_z - near_z),
        r4c4: 1.0 
    }
}
