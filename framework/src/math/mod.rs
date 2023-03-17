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

#[inline]
pub fn orthographic_lh_zo(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32
) -> Mat4x4 {
    Mat4x4 {
        r1c1: 2.0 / (right - left),
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 2.0 / (top - bottom),
        r2c3: 0.0,
        r2c4: 0.0,
        
        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: 1.0 / (far - near),
        r3c4: 0.0,

        r4c1: - (right + left) / (right - left),
        r4c2: - (top + bottom) / (top - bottom),
        r4c3: - near / (far - near),
        r4c4: 1.0
    }
}

#[inline]
pub fn orthographic_lh_no(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32
) -> Mat4x4 {
    Mat4x4 {
        r1c1: 2.0 / (right - left),
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 2.0 / (top - bottom),
        r2c3: 0.0,
        r2c4: 0.0,

        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: 2.0 / (far - near),
        r3c4: 0.0,

        r4c1: - (right + left) / (right - left),
        r4c2: - (top + bottom) / (top - bottom),
        r4c3: - (far + near) / (far - near),
        r4c4: 1.0
    }
}

#[inline]
pub fn orthographic_rh_zo(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32
) -> Mat4x4 {
    Mat4x4 {
        r1c1: 2.0 / (right - left),
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 2.0 / (top - bottom),
        r2c3: 0.0,
        r2c4: 0.0,
        
        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: -1.0 / (far - near),
        r3c4: 0.0,

        r4c1: - (right + left) / (right - left),
        r4c2: - (top + bottom) / (top - bottom),
        r4c3: - near / (far - near),
        r4c4: 1.0
    }
}

#[inline]
pub fn orthographic_rh_no(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32
) -> Mat4x4 {
    Mat4x4 {
        r1c1: 2.0 / (right - left),
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 2.0 / (top - bottom),
        r2c3: 0.0,
        r2c4: 0.0,
        
        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: -2.0 / (far - near),
        r3c4: 0.0,

        r4c1: - (right + left) / (right - left),
        r4c2: - (top + bottom) / (top - bottom),
        r4c3: - (far + near) / (far - near),
        r4c4: 1.0
    }
}

#[inline]
pub fn perspective_rh_zo(
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32
) -> Mat4x4 {
    assert!(!(aspect.abs() <= f32::EPSILON), "aspect must be greater than EPSILON.");

    let tan_half_fovy = (fovy * 0.5).tan();

    Mat4x4 {
        r1c1: 1.0 / (aspect * tan_half_fovy),
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 1.0 / tan_half_fovy,
        r2c3: 0.0,
        r2c4: 0.0,

        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: far / (near - far),
        r3c4: -1.0,

        r4c1: 0.0,
        r4c2: 0.0,
        r4c3: - (far * near) / (far - near),
        r4c4: 0.0
    }
}

#[inline]
pub fn perspective_rh_no(
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32
) -> Mat4x4 {
    assert!(!(aspect.abs() <= f32::EPSILON), "aspect must be greater than EPSILON.");

    let tan_half_fovy = (fovy * 0.5).tan();

    Mat4x4 {
        r1c1: 1.0 / (aspect * tan_half_fovy),
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 1.0 / tan_half_fovy,
        r2c3: 0.0,
        r2c4: 0.0,

        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: - (far + near) / (far - near),
        r3c4: -1.0,

        r4c1: 0.0,
        r4c2: 0.0,
        r4c3: - (2.0 * far * near) / (far - near),
        r4c4: 0.0
    }
}

#[inline]
pub fn perspective_lh_zo(
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32
) -> Mat4x4 {
    assert!(!(aspect.abs() <= f32::EPSILON), "aspect must be greater than EPSILON.");

    let tan_half_fovy = (fovy * 0.5).tan();

    Mat4x4 {
        r1c1: 1.0 / (aspect * tan_half_fovy),
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 1.0 / tan_half_fovy,
        r2c3: 0.0,
        r2c4: 0.0,

        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: far / (far - near),
        r3c4: 1.0,

        r4c1: 0.0,
        r4c2: 0.0,
        r4c3: - (far * near) / (far - near),
        r4c4: 0.0
    }
}

#[inline]
pub fn perspective_lh_no(
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32
) -> Mat4x4 {
    assert!(!(aspect.abs() <= f32::EPSILON), "aspect must be greater than EPSILON.");

    let tan_half_fovy = (fovy * 0.5).tan();

    Mat4x4 {
        r1c1: 1.0 / (aspect * tan_half_fovy),
        r1c2: 0.0,
        r1c3: 0.0,
        r1c4: 0.0,

        r2c1: 0.0,
        r2c2: 1.0 / tan_half_fovy,
        r2c3: 0.0,
        r2c4: 0.0,

        r3c1: 0.0,
        r3c2: 0.0,
        r3c3: (far + near) / (far - near),
        r3c4: 1.0,

        r4c1: 0.0,
        r4c2: 0.0,
        r4c3: - (2.0 * far * near) / (far - near),
        r4c4: 0.0
    }
}
