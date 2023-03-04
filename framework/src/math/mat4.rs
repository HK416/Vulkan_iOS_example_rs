use std::cmp;
use std::ops;
use std::fmt;
use super::mat3::Mat3x3;
use super::quat::Quat;
use super::vec4::Vec4;

/// 4by4 matrix.
/// - row major
/// - pre-multiplication
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Mat4x4 {
    pub r1c1: f32, pub r1c2: f32, pub r1c3: f32, pub r1c4: f32,
    pub r2c1: f32, pub r2c2: f32, pub r2c3: f32, pub r2c4: f32,
    pub r3c1: f32, pub r3c2: f32, pub r3c3: f32, pub r3c4: f32,
    pub r4c1: f32, pub r4c2: f32, pub r4c3: f32, pub r4c4: f32,
}

impl Mat4x4 {
    /// matrix with all elements `0`.
    pub const ZERO: Self = Self::new_scalar(0.0);

    /// identity matrix.
    pub const IDENTITY: Self = Self::new_rows(
        Vec4::new_vector(1.0, 0.0, 0.0, 0.0), 
        Vec4::new_vector(0.0, 1.0, 0.0, 0.0), 
        Vec4::new_vector(0.0, 0.0, 1.0, 0.0), 
        Vec4::new_vector(0.0, 0.0, 0.0, 1.0)
    );

    /// create a matrix with the values of the given elements.
    #[inline]
    pub const fn new(
        r1c1: f32, r1c2: f32, r1c3: f32, r1c4: f32,
        r2c1: f32, r2c2: f32, r2c3: f32, r2c4: f32,
        r3c1: f32, r3c2: f32, r3c3: f32, r3c4: f32,
        r4c1: f32, r4c2: f32, r4c3: f32, r4c4: f32 
    ) -> Self {
        Self {
            r1c1, r1c2, r1c3, r1c4,
            r2c1, r2c2, r2c3, r2c4,
            r3c1, r3c2, r3c3, r3c4,
            r4c1, r4c2, r4c3, r4c4
        }
    }

    /// create a matrix with the given scalar value.
    #[inline]
    pub const fn new_scalar(scalar: f32) -> Self {
        Self {
            r1c1: scalar, r1c2: scalar, r1c3: scalar, r1c4: scalar,
            r2c1: scalar, r2c2: scalar, r2c3: scalar, r2c4: scalar,
            r3c1: scalar, r3c2: scalar, r3c3: scalar, r3c4: scalar,
            r4c1: scalar, r4c2: scalar, r4c3: scalar, r4c4: scalar 
        }
    }

    /// create a matrix with given row-major vectors.
    #[inline]
    pub const fn new_rows(row1: Vec4, row2: Vec4, row3: Vec4, row4: Vec4) -> Self {
        Self {
            r1c1: row1.x, r1c2: row1.y, r1c3: row1.z, r1c4: row1.w,
            r2c1: row2.x, r2c2: row2.y, r2c3: row2.z, r2c4: row2.w,
            r3c1: row3.x, r3c2: row3.y, r3c3: row3.z, r3c4: row3.w,
            r4c1: row4.x, r4c2: row4.y, r4c3: row4.z, r4c4: row4.w
        }
    }

    /// create a matrix with given quaternion.
    pub fn from_quat(quat: Quat) -> Self {
        Self {
            r1c1: 1.0 - 2.0 * quat.y * quat.y - 2.0 * quat.z * quat.z,
            r1c2: 2.0 * quat.x * quat.y + 2.0 * quat.z * quat.w,
            r1c3: 2.0 * quat.x * quat.z - 2.0 * quat.y * quat.w,
            r1c4: 0.0,
            
            r2c1: 2.0 * quat.x * quat.y - 2.0 * quat.z * quat.w,
            r2c2: 1.0 - 2.0 * quat.x * quat.x - 2.0 * quat.z * quat.z,
            r2c3: 2.0 * quat.y * quat.z + 2.0 * quat.x * quat.w,
            r2c4: 0.0,

            r3c1: 2.0 * quat.x * quat.z + 2.0 * quat.y * quat.w,
            r3c2: 2.0 * quat.y * quat.z - 2.0 * quat.x * quat.w,
            r3c3: 1.0 - 2.0 * quat.x * quat.x - 2.0 * quat.y * quat.y,
            r3c4: 0.0,
            
            r4c1: 0.0,
            r4c2: 0.0,
            r4c3: 0.0,
            r4c4: 1.0 
        }
    }

    /// convert a matrix to an quaternion.
    #[inline]
    pub fn into_quat(self) -> Quat {
        Quat::from_matrix4x4(self)
    }

    #[inline]
    pub fn add_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 + rhs, r1c2: self.r1c2 + rhs, r1c3: self.r1c3 + rhs, r1c4: self.r1c4 + rhs,
            r2c1: self.r2c1 + rhs, r2c2: self.r2c2 + rhs, r2c3: self.r2c3 + rhs, r2c4: self.r2c4 + rhs,
            r3c1: self.r3c1 + rhs, r3c2: self.r3c2 + rhs, r3c3: self.r3c3 + rhs, r3c4: self.r3c4 + rhs,
            r4c1: self.r4c1 + rhs, r4c2: self.r4c2 + rhs, r4c3: self.r4c3 + rhs, r4c4: self.r4c4 + rhs 
        }
    }

    #[inline]
    pub fn add_assign_scalar(&mut self, rhs: f32) {
        *self = self.add_scalar(rhs)
    }

    #[inline]
    pub fn add_matrix4x4(self, rhs: Self) -> Self {
        Self {
            r1c1: self.r1c1 + rhs.r1c1, r1c2: self.r1c2 + rhs.r1c2, r1c3: self.r1c3 + rhs.r1c3, r1c4: self.r1c4 + rhs.r1c4,
            r2c1: self.r2c1 + rhs.r2c1, r2c2: self.r2c2 + rhs.r2c2, r2c3: self.r2c3 + rhs.r2c3, r2c4: self.r2c4 + rhs.r2c4,
            r3c1: self.r3c1 + rhs.r3c1, r3c2: self.r3c2 + rhs.r3c2, r3c3: self.r3c3 + rhs.r3c3, r3c4: self.r3c4 + rhs.r3c4,
            r4c1: self.r4c1 + rhs.r4c1, r4c2: self.r4c2 + rhs.r4c2, r4c3: self.r4c3 + rhs.r4c3, r4c4: self.r4c4 + rhs.r4c4 
        }
    }

    #[inline]
    pub fn add_assign_matrix4x4(&mut self, rhs: Self) {
        *self = self.add_matrix4x4(rhs)
    }

    #[inline]
    pub fn sub_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 - rhs, r1c2: self.r1c2 - rhs, r1c3: self.r1c3 - rhs, r1c4: self.r1c4 - rhs,
            r2c1: self.r2c1 - rhs, r2c2: self.r2c2 - rhs, r2c3: self.r2c3 - rhs, r2c4: self.r2c4 - rhs,
            r3c1: self.r3c1 - rhs, r3c2: self.r3c2 - rhs, r3c3: self.r3c3 - rhs, r3c4: self.r3c4 - rhs,
            r4c1: self.r4c1 - rhs, r4c2: self.r4c2 - rhs, r4c3: self.r4c3 - rhs, r4c4: self.r4c4 - rhs 
        }
    }

    #[inline]
    pub fn sub_assign_scalar(&mut self, rhs: f32) {
        *self = self.sub_scalar(rhs)
    }

    #[inline]
    pub fn sub_matrix4x4(self, rhs: Self) -> Self {
        Self {
            r1c1: self.r1c1 - rhs.r1c1, r1c2: self.r1c2 - rhs.r1c2, r1c3: self.r1c3 - rhs.r1c3, r1c4: self.r1c4 - rhs.r1c4,
            r2c1: self.r2c1 - rhs.r2c1, r2c2: self.r2c2 - rhs.r2c2, r2c3: self.r2c3 - rhs.r2c3, r2c4: self.r2c4 - rhs.r2c4,
            r3c1: self.r3c1 - rhs.r3c1, r3c2: self.r3c2 - rhs.r3c2, r3c3: self.r3c3 - rhs.r3c3, r3c4: self.r3c4 - rhs.r3c4,
            r4c1: self.r4c1 - rhs.r4c1, r4c2: self.r4c2 - rhs.r4c2, r4c3: self.r4c3 - rhs.r4c3, r4c4: self.r4c4 - rhs.r4c4 
        }
    }

    #[inline]
    pub fn sub_assign_matrix4x4(&mut self, rhs: Self) {
        *self = self.sub_matrix4x4(rhs)
    }

    #[inline]
    pub fn mul_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 * rhs, r1c2: self.r1c2 * rhs, r1c3: self.r1c3 * rhs, r1c4: self.r1c4 * rhs,
            r2c1: self.r2c1 * rhs, r2c2: self.r2c2 * rhs, r2c3: self.r2c3 * rhs, r2c4: self.r2c4 * rhs,
            r3c1: self.r3c1 * rhs, r3c2: self.r3c2 * rhs, r3c3: self.r3c3 * rhs, r3c4: self.r3c4 * rhs,
            r4c1: self.r4c1 * rhs, r4c2: self.r4c2 * rhs, r4c3: self.r4c3 * rhs, r4c4: self.r4c4 * rhs 
        }
    }

    #[inline]
    pub fn mul_assign_scalar(&mut self, rhs: f32) {
        *self = self.mul_scalar(rhs)
    }

    #[inline]
    pub fn mul_matrix4x4(self, rhs: Self) -> Self {
        Mat4x4 {
            r1c1: self.r1c1 * rhs.r1c1 + self.r1c2 * rhs.r2c1 + self.r1c3 * rhs.r3c1 + self.r1c4 * rhs.r4c1,
            r2c1: self.r2c1 * rhs.r1c1 + self.r2c2 * rhs.r2c1 + self.r2c3 * rhs.r3c1 + self.r2c4 * rhs.r4c1,
            r3c1: self.r3c1 * rhs.r1c1 + self.r3c2 * rhs.r2c1 + self.r3c3 * rhs.r3c1 + self.r3c4 * rhs.r4c1,
            r4c1: self.r4c1 * rhs.r1c1 + self.r4c2 * rhs.r2c1 + self.r4c3 * rhs.r3c1 + self.r4c4 * rhs.r4c1,

            r1c2: self.r1c1 * rhs.r1c2 + self.r1c2 * rhs.r2c2 + self.r1c3 * rhs.r3c2 + self.r1c4 * rhs.r4c2,
            r2c2: self.r2c1 * rhs.r1c2 + self.r2c2 * rhs.r2c2 + self.r2c3 * rhs.r3c2 + self.r2c4 * rhs.r4c2,
            r3c2: self.r3c1 * rhs.r1c2 + self.r3c2 * rhs.r2c2 + self.r3c3 * rhs.r3c2 + self.r3c4 * rhs.r4c2,
            r4c2: self.r4c1 * rhs.r1c2 + self.r4c2 * rhs.r2c2 + self.r4c3 * rhs.r3c2 + self.r4c4 * rhs.r4c2,
            
            r1c3: self.r1c1 * rhs.r1c3 + self.r1c2 * rhs.r2c3 + self.r1c3 * rhs.r3c3 + self.r1c4 * rhs.r4c3,
            r2c3: self.r2c1 * rhs.r1c3 + self.r2c2 * rhs.r2c3 + self.r2c3 * rhs.r3c3 + self.r2c4 * rhs.r4c3,
            r3c3: self.r3c1 * rhs.r1c3 + self.r3c2 * rhs.r2c3 + self.r3c3 * rhs.r3c3 + self.r3c4 * rhs.r4c3,
            r4c3: self.r4c1 * rhs.r1c3 + self.r4c2 * rhs.r2c3 + self.r4c3 * rhs.r3c3 + self.r4c4 * rhs.r4c3,

            r1c4: self.r1c1 * rhs.r1c4 + self.r1c2 * rhs.r2c4 + self.r1c3 * rhs.r3c4 + self.r1c4 * rhs.r4c4,
            r2c4: self.r2c1 * rhs.r1c4 + self.r2c2 * rhs.r2c4 + self.r2c3 * rhs.r3c4 + self.r2c4 * rhs.r4c4,
            r3c4: self.r3c1 * rhs.r1c4 + self.r3c2 * rhs.r2c4 + self.r3c3 * rhs.r3c4 + self.r3c4 * rhs.r4c4,
            r4c4: self.r4c1 * rhs.r1c4 + self.r4c2 * rhs.r2c4 + self.r4c3 * rhs.r3c4 + self.r4c4 * rhs.r4c4 
        }
    }

    #[inline]
    pub fn mul_assign_matrix4x4(&mut self, rhs: Self) {
        *self = self.mul_matrix4x4(rhs)
    }

    #[inline]
    pub fn div_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 / rhs, r1c2: self.r1c2 / rhs, r1c3: self.r1c3 / rhs, r1c4: self.r1c4 / rhs,
            r2c1: self.r2c1 / rhs, r2c2: self.r2c2 / rhs, r2c3: self.r2c3 / rhs, r2c4: self.r2c4 / rhs,
            r3c1: self.r3c1 / rhs, r3c2: self.r3c2 / rhs, r3c3: self.r3c3 / rhs, r3c4: self.r3c4 / rhs,
            r4c1: self.r4c1 / rhs, r4c2: self.r4c2 / rhs, r4c3: self.r4c3 / rhs, r4c4: self.r4c4 / rhs 
        }
    }

    #[inline]
    pub fn div_assign_scalar(&mut self, rhs: f32) {
        *self = self.div_scalar(rhs)
    }

    /// return transpose matrix.
    #[inline]
    pub fn transpose(&self) -> Self {
        Self {
            r1c1: self.r1c1, r1c2: self.r2c1, r1c3: self.r3c1, r1c4: self.r4c1,
            r2c1: self.r1c2, r2c2: self.r2c2, r2c3: self.r3c2, r2c4: self.r4c2,
            r3c1: self.r1c3, r3c2: self.r2c3, r3c3: self.r3c3, r3c4: self.r4c3,
            r4c1: self.r1c4, r4c2: self.r2c4, r4c3: self.r3c4, r4c4: self.r4c4 
        }
    }

    /// return a determinant of the matrix.
    #[inline]
    pub fn determinant(&self) -> f32 {
        self.r1c1 * self.r2c2 * self.r3c3 * self.r4c4 + self.r1c1 * self.r2c3 * self.r3c4 * self.r4c2 + self.r1c1 * self.r2c4 * self.r3c2 * self.r4c3
        - self.r1c1 * self.r2c4 * self.r3c3 * self.r4c2 - self.r1c1 * self.r2c3 * self.r3c2 * self.r4c4 - self.r1c1 * self.r2c2 * self.r3c4 * self.r4c3
        - self.r1c2 * self.r2c1 * self.r3c3 * self.r4c4 - self.r1c3 * self.r2c1 * self.r3c4 * self.r4c2 - self.r1c4 * self.r2c1 * self.r3c2 * self.r4c3
        + self.r1c4 * self.r2c1 * self.r3c3 * self.r4c2 + self.r1c3 * self.r2c1 * self.r3c2 * self.r4c4 + self.r1c2 * self.r2c1 * self.r3c4 * self.r4c3
        + self.r1c2 * self.r2c3 * self.r3c1 * self.r4c4 + self.r1c3 * self.r2c4 * self.r3c1 * self.r4c2 + self.r1c4 * self.r2c2 * self.r3c1 * self.r4c3
        - self.r1c4 * self.r2c3 * self.r3c1 * self.r4c2 - self.r1c3 * self.r2c2 * self.r3c1 * self.r4c4 - self.r1c2 * self.r2c4 * self.r3c1 * self.r4c3
        - self.r1c2 * self.r2c3 * self.r3c4 * self.r4c1 - self.r1c3 * self.r2c4 * self.r3c2 * self.r4c1 - self.r1c4 * self.r2c2 * self.r3c3 * self.r4c1
        + self.r1c4 * self.r2c3 * self.r3c2 * self.r4c1 + self.r1c3 * self.r2c2 * self.r3c4 * self.r4c1 + self.r1c2 * self.r2c4 * self.r3c3 * self.r4c1
    }

    /// return inverse matrix.
    #[inline]
    pub fn inverse(&self) -> Self {
        let mt = self.transpose();
        let det = self.determinant();

        let cof_r1c1 = 1.0 * minor_matrix(&mt, 1, 1).determinant();
        let cof_r1c2 = -1.0 * minor_matrix(&mt, 1, 2).determinant();
        let cof_r1c3 = 1.0 * minor_matrix(&mt, 1, 3).determinant();
        let cof_r1c4 = -1.0 * minor_matrix(&mt, 1, 4).determinant();

        let cof_r2c1 = -1.0 * minor_matrix(&mt, 2, 1).determinant();
        let cof_r2c2 = 1.0 * minor_matrix(&mt, 2, 2).determinant();
        let cof_r2c3 = -1.0 * minor_matrix(&mt, 2, 3).determinant();
        let cof_r2c4 = 1.0 * minor_matrix(&mt, 2, 4).determinant();

        let cof_r3c1 = 1.0 * minor_matrix(&mt, 3, 1).determinant();
        let cof_r3c2 = -1.0 * minor_matrix(&mt, 3, 2).determinant();
        let cof_r3c3 = 1.0 * minor_matrix(&mt, 3, 3).determinant();
        let cof_r3c4 = -1.0 * minor_matrix(&mt, 3, 4).determinant();

        let cof_r4c1 = -1.0 * minor_matrix(&mt, 4, 1).determinant();
        let cof_r4c2 = 1.0 * minor_matrix(&mt, 4, 2).determinant();
        let cof_r4c3 = -1.0 * minor_matrix(&mt, 4, 3).determinant();
        let cof_r4c4 = 1.0 * minor_matrix(&mt, 4, 4).determinant();

        Self {
            r1c1: cof_r1c1 / det, r1c2: cof_r1c2 / det, r1c3: cof_r1c3 / det, r1c4: cof_r1c4 / det,
            r2c1: cof_r2c1 / det, r2c2: cof_r2c2 / det, r2c3: cof_r2c3 / det, r2c4: cof_r2c4 / det,
            r3c1: cof_r3c1 / det, r3c2: cof_r3c2 / det, r3c3: cof_r3c3 / det, r3c4: cof_r3c4 / det,
            r4c1: cof_r4c1 / det, r4c2: cof_r4c2 / det, r4c3: cof_r4c3 / det, r4c4: cof_r4c4 / det,
        }
    }

    /// return `None` if matrix cannot be create inverse matrix.
    #[inline]
    pub fn try_inverse(&self) -> Option<Self> {
        let mt = self.transpose();
        let det = self.determinant();

        if det.abs() > f32::EPSILON {
            let cof_r1c1 = 1.0 * minor_matrix(&mt, 1, 1).determinant();
            let cof_r1c2 = -1.0 * minor_matrix(&mt, 1, 2).determinant();
            let cof_r1c3 = 1.0 * minor_matrix(&mt, 1, 3).determinant();
            let cof_r1c4 = -1.0 * minor_matrix(&mt, 1, 4).determinant();
            
            let cof_r2c1 = -1.0 * minor_matrix(&mt, 2, 1).determinant();
            let cof_r2c2 = 1.0 * minor_matrix(&mt, 2, 2).determinant();
            let cof_r2c3 = -1.0 * minor_matrix(&mt, 2, 3).determinant();
            let cof_r2c4 = 1.0 * minor_matrix(&mt, 2, 4).determinant();
            
            let cof_r3c1 = 1.0 * minor_matrix(&mt, 3, 1).determinant();
            let cof_r3c2 = -1.0 * minor_matrix(&mt, 3, 2).determinant();
            let cof_r3c3 = 1.0 * minor_matrix(&mt, 3, 3).determinant();
            let cof_r3c4 = -1.0 * minor_matrix(&mt, 3, 4).determinant();
            
            let cof_r4c1 = -1.0 * minor_matrix(&mt, 4, 1).determinant();
            let cof_r4c2 = 1.0 * minor_matrix(&mt, 4, 2).determinant();
            let cof_r4c3 = -1.0 * minor_matrix(&mt, 4, 3).determinant();
            let cof_r4c4 = 1.0 * minor_matrix(&mt, 4, 4).determinant();
            
            return Some(Self {
                r1c1: cof_r1c1 / det, r1c2: cof_r1c2 / det, r1c3: cof_r1c3 / det, r1c4: cof_r1c4 / det,
                r2c1: cof_r2c1 / det, r2c2: cof_r2c2 / det, r2c3: cof_r2c3 / det, r2c4: cof_r2c4 / det,
                r3c1: cof_r3c1 / det, r3c2: cof_r3c2 / det, r3c3: cof_r3c3 / det, r3c4: cof_r3c4 / det,
                r4c1: cof_r4c1 / det, r4c2: cof_r4c2 / det, r4c3: cof_r4c3 / det, r4c4: cof_r4c4 / det,
            });
        }
        return None;
    }

    /// return `true` if any element of the matrix has the value of infinity.
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.r1c1.is_infinite() | self.r1c2.is_infinite() | self.r1c3.is_infinite() | self.r1c4.is_infinite()
        | self.r2c1.is_infinite() | self.r2c2.is_infinite() | self.r2c3.is_infinite() | self.r2c4.is_infinite()
        | self.r3c1.is_infinite() | self.r3c2.is_infinite() | self.r3c3.is_infinite() | self.r3c4.is_infinite()
        | self.r4c1.is_infinite() | self.r4c2.is_infinite() | self.r4c3.is_infinite() | self.r4c4.is_infinite()
    }

    /// return `true` if all elements of the matrix are neither infinite nor NaN.
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.r1c1.is_finite() & self.r1c2.is_finite() & self.r1c3.is_finite() & self.r1c4.is_finite()
        & self.r2c1.is_finite() & self.r2c2.is_finite() & self.r2c3.is_finite() & self.r2c4.is_finite()
        & self.r3c1.is_finite() & self.r3c2.is_finite() & self.r3c3.is_finite() & self.r3c4.is_finite()
        & self.r4c1.is_finite() & self.r4c2.is_finite() & self.r4c3.is_finite() & self.r4c4.is_finite()
    }

    /// return `true` if any element of the matrix has the value of NaN.
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.r1c1.is_nan() | self.r1c2.is_nan() | self.r1c3.is_nan() | self.r1c4.is_nan()
        | self.r2c1.is_nan() | self.r2c2.is_nan() | self.r2c3.is_nan() | self.r2c4.is_nan()
        | self.r3c1.is_nan() | self.r3c2.is_nan() | self.r3c3.is_nan() | self.r3c4.is_nan()
        | self.r4c1.is_nan() | self.r4c2.is_nan() | self.r4c3.is_nan() | self.r4c4.is_nan()
    }

    /// return `true` if the two matrices are equal.
    #[inline]
    pub fn equal(&self, other: &Self) -> bool {
        let mut flag = true;
        for &num in (*self - *other).as_ref().iter() {
            flag &= num.abs() <= f32::EPSILON
        }
        return flag;
    }

    /// return the smaller of the elements of two matrices.
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self { 
            r1c1: self.r1c1.min(other.r1c1), r1c2: self.r1c2.min(other.r1c2), r1c3: self.r1c3.min(other.r1c3), r1c4: self.r1c4.min(other.r1c4),
            r2c1: self.r2c1.min(other.r2c1), r2c2: self.r2c2.min(other.r2c2), r2c3: self.r2c3.min(other.r2c3), r2c4: self.r2c4.min(other.r2c4),
            r3c1: self.r3c1.min(other.r3c1), r3c2: self.r3c2.min(other.r3c2), r3c3: self.r3c3.min(other.r3c3), r3c4: self.r3c4.min(other.r3c4),
            r4c1: self.r4c1.min(other.r4c1), r4c2: self.r4c2.min(other.r4c2), r4c3: self.r4c3.min(other.r4c3), r4c4: self.r4c4.min(other.r4c4),
        }
    }

    /// return the greater of the elements of two matrices.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self { 
            r1c1: self.r1c1.max(other.r1c1), r1c2: self.r1c2.max(other.r1c2), r1c3: self.r1c3.max(other.r1c3), r1c4: self.r1c4.max(other.r1c4),
            r2c1: self.r2c1.max(other.r2c1), r2c2: self.r2c2.max(other.r2c2), r2c3: self.r2c3.max(other.r2c3), r2c4: self.r2c4.max(other.r2c4),
            r3c1: self.r3c1.max(other.r3c1), r3c2: self.r3c2.max(other.r3c2), r3c3: self.r3c3.max(other.r3c3), r3c4: self.r3c4.max(other.r3c4),
            r4c1: self.r4c1.max(other.r4c1), r4c2: self.r4c2.max(other.r4c2), r4c3: self.r4c3.max(other.r4c3), r4c4: self.r4c4.max(other.r4c4),
        }
    }

    /// round up the decimal places of the elements of a matrix.
    #[inline]
    pub fn ceil(self) -> Self {
        Self { 
            r1c1: self.r1c1.ceil(), r1c2: self.r1c2.ceil(), r1c3: self.r1c3.ceil(), r1c4: self.r1c4.ceil(),
            r2c1: self.r2c1.ceil(), r2c2: self.r2c2.ceil(), r2c3: self.r2c3.ceil(), r2c4: self.r2c4.ceil(),
            r3c1: self.r3c1.ceil(), r3c2: self.r3c2.ceil(), r3c3: self.r3c3.ceil(), r3c4: self.r3c4.ceil(),
            r4c1: self.r4c1.ceil(), r4c2: self.r4c2.ceil(), r4c3: self.r4c3.ceil(), r4c4: self.r4c4.ceil(),
        }
    }

    /// round down the decimal places of the elements of a matrix.
    #[inline]
    pub fn floor(self) -> Self {
        Self { 
            r1c1: self.r1c1.floor(), r1c2: self.r1c2.floor(), r1c3: self.r1c3.floor(), r1c4: self.r1c4.floor(),
            r2c1: self.r2c1.floor(), r2c2: self.r2c2.floor(), r2c3: self.r2c3.floor(), r2c4: self.r2c4.floor(),
            r3c1: self.r3c1.floor(), r3c2: self.r3c2.floor(), r3c3: self.r3c3.floor(), r3c4: self.r3c4.floor(),
            r4c1: self.r4c1.floor(), r4c2: self.r4c2.floor(), r4c3: self.r4c3.floor(), r4c4: self.r4c4.floor(),
        }
    }

    /// round the decimal places of the elements of a matrix.
    #[inline]
    pub fn round(self) -> Self {
        Self { 
            r1c1: self.r1c1.round(), r1c2: self.r1c2.round(), r1c3: self.r1c3.round(), r1c4: self.r1c4.round(),
            r2c1: self.r2c1.round(), r2c2: self.r2c2.round(), r2c3: self.r2c3.round(), r2c4: self.r2c4.round(),
            r3c1: self.r3c1.round(), r3c2: self.r3c2.round(), r3c3: self.r3c3.round(), r3c4: self.r3c4.round(),
            r4c1: self.r4c1.round(), r4c2: self.r4c2.round(), r4c3: self.r4c3.round(), r4c4: self.r4c4.round(),
        }
    }
}


impl ops::Add<Mat4x4> for f32 {
    type Output = Mat4x4;
    #[inline]
    fn add(self, rhs: Mat4x4) -> Self::Output {
        Mat4x4 {
            r1c1: self + rhs.r1c1, r1c2: self + rhs.r1c2, r1c3: self + rhs.r1c3, r1c4: self + rhs.r1c4,
            r2c1: self + rhs.r2c1, r2c2: self + rhs.r2c2, r2c3: self + rhs.r2c3, r2c4: self + rhs.r2c4,
            r3c1: self + rhs.r3c1, r3c2: self + rhs.r3c2, r3c3: self + rhs.r3c3, r3c4: self + rhs.r3c4,
            r4c1: self + rhs.r4c1, r4c2: self + rhs.r4c2, r4c3: self + rhs.r4c3, r4c4: self + rhs.r4c4 
        }
    }
}

impl ops::Add<f32> for Mat4x4 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        self.add_scalar(rhs)
    }
}

impl ops::AddAssign<f32> for Mat4x4 {
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.add_assign_scalar(rhs)
    }
}

impl ops::Add<Self> for Mat4x4 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.add_matrix4x4(rhs)
    }
}

impl ops::AddAssign<Self> for Mat4x4 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign_matrix4x4(rhs)
    }
}

impl ops::Sub<Mat4x4> for f32 {
    type Output = Mat4x4;
    #[inline]
    fn sub(self, rhs: Mat4x4) -> Self::Output {
        Mat4x4 {
            r1c1: self - rhs.r1c1, r1c2: self - rhs.r1c2, r1c3: self - rhs.r1c3, r1c4: self - rhs.r1c4,
            r2c1: self - rhs.r2c1, r2c2: self - rhs.r2c2, r2c3: self - rhs.r2c3, r2c4: self - rhs.r2c4,
            r3c1: self - rhs.r3c1, r3c2: self - rhs.r3c2, r3c3: self - rhs.r3c3, r3c4: self - rhs.r3c4,
            r4c1: self - rhs.r4c1, r4c2: self - rhs.r4c2, r4c3: self - rhs.r4c3, r4c4: self - rhs.r4c4 
        }
    }
}

impl ops::Sub<f32> for Mat4x4 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        self.sub_scalar(rhs)
    }
}

impl ops::SubAssign<f32> for Mat4x4 {
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.sub_assign_scalar(rhs)
    }
}

impl ops::Sub<Self> for Mat4x4 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_matrix4x4(rhs)
    }
}

impl ops::SubAssign<Self> for Mat4x4 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign_matrix4x4(rhs)
    }
}

impl ops::Neg for Mat4x4 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            r1c1: self.r1c1.neg(), r1c2: self.r1c2.neg(), r1c3: self.r1c3.neg(), r1c4: self.r1c4.neg(),
            r2c1: self.r2c1.neg(), r2c2: self.r2c2.neg(), r2c3: self.r2c3.neg(), r2c4: self.r2c4.neg(),
            r3c1: self.r3c1.neg(), r3c2: self.r3c2.neg(), r3c3: self.r3c3.neg(), r3c4: self.r3c4.neg(),
            r4c1: self.r4c1.neg(), r4c2: self.r4c2.neg(), r4c3: self.r4c3.neg(), r4c4: self.r4c4.neg()
        }
    }
}

impl ops::Mul<Mat4x4> for f32 {
    type Output = Mat4x4;
    #[inline]
    fn mul(self, rhs: Mat4x4) -> Self::Output {
        Mat4x4 {
            r1c1: self * rhs.r1c1, r1c2: self * rhs.r1c2, r1c3: self * rhs.r1c3, r1c4: self * rhs.r1c4,
            r2c1: self * rhs.r2c1, r2c2: self * rhs.r2c2, r2c3: self * rhs.r2c3, r2c4: self * rhs.r2c4,
            r3c1: self * rhs.r3c1, r3c2: self * rhs.r3c2, r3c3: self * rhs.r3c3, r3c4: self * rhs.r3c4,
            r4c1: self * rhs.r4c1, r4c2: self * rhs.r4c2, r4c3: self * rhs.r4c3, r4c4: self * rhs.r4c4 
        }
    }
}

impl ops::Mul<f32> for Mat4x4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        self.mul_scalar(rhs)
    }
}

impl ops::MulAssign<f32> for Mat4x4 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.mul_assign_scalar(rhs)
    }
}

impl ops::Mul<Self> for Mat4x4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_matrix4x4(rhs)
    }
}

impl ops::MulAssign<Self> for Mat4x4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.mul_assign_matrix4x4(rhs)
    }
}

impl ops::Div<Mat4x4> for f32 {
    type Output = Mat4x4;
    #[inline]
    fn div(self, rhs: Mat4x4) -> Self::Output {
        Mat4x4 {
            r1c1: self / rhs.r1c1, r1c2: self / rhs.r1c2, r1c3: self / rhs.r1c3, r1c4: self / rhs.r1c4,
            r2c1: self / rhs.r2c1, r2c2: self / rhs.r2c2, r2c3: self / rhs.r2c3, r2c4: self / rhs.r2c4,
            r3c1: self / rhs.r3c1, r3c2: self / rhs.r3c2, r3c3: self / rhs.r3c3, r3c4: self / rhs.r3c4,
            r4c1: self / rhs.r4c1, r4c2: self / rhs.r4c2, r4c3: self / rhs.r4c3, r4c4: self / rhs.r4c4 
        }
    }
}

impl ops::Div<f32> for Mat4x4 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        self.div_scalar(rhs)
    }
}

impl ops::DivAssign<f32> for Mat4x4 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.div_assign_scalar(rhs)
    }
}

impl cmp::PartialEq<Self> for Mat4x4 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

impl AsRef<[f32; 16]> for Mat4x4 {
    #[inline]
    fn as_ref(&self) -> &[f32; 16] {
        unsafe { &*(self as *const Self as *const [f32; 16]) }
    }
}

impl AsMut<[f32; 16]> for Mat4x4 {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 16] {
        unsafe { &mut *(self as *mut Self as *mut [f32; 16]) }
    }
}

impl fmt::Display for Mat4x4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "[({}, {}, {}, {}), ({}, {}, {}, {}), ({}, {}, {}, {}), ({}, {}, {}, {})]",
            self.r1c1, self.r1c2, self.r1c3, self.r1c4,
            self.r2c1, self.r2c2, self.r2c3, self.r2c4,
            self.r3c1, self.r3c2, self.r3c3, self.r3c4,
            self.r4c1, self.r4c2, self.r4c3, self.r4c4
        )
    }
}

#[inline]
fn minor_matrix(mat: &Mat4x4, row: usize, col: usize) -> Mat3x3 {
    debug_assert!(0 < row && row <= 4, "row out of range!");
    debug_assert!(0 < col && col <= 4, "column out of range!");
    match (row, col) {
        (1, 1) => {
            Mat3x3::new(
                mat.r2c2, mat.r2c3, mat.r2c4, 
                mat.r3c2, mat.r3c3, mat.r3c4, 
                mat.r4c2, mat.r4c3, mat.r4c4
            )
        },
        (1, 2) => {
            Mat3x3::new(
                mat.r2c1, mat.r2c3, mat.r2c4, 
                mat.r3c1, mat.r3c3, mat.r3c4, 
                mat.r4c1, mat.r4c3, mat.r4c4
            )
        },
        (1, 3) => {
            Mat3x3::new(
                mat.r2c1, mat.r2c2, mat.r2c4, 
                mat.r3c1, mat.r3c2, mat.r3c4, 
                mat.r4c1, mat.r4c2, mat.r4c4
            )
        },
        (1, 4) => {
            Mat3x3::new(
                mat.r2c1, mat.r2c2, mat.r2c3, 
                mat.r3c1, mat.r3c2, mat.r3c3, 
                mat.r4c1, mat.r4c2, mat.r4c3
            )
        },
        (2, 1) => {
            Mat3x3::new(
                mat.r1c2, mat.r1c3, mat.r1c4,
                mat.r3c2, mat.r3c3, mat.r3c4, 
                mat.r4c2, mat.r4c3, mat.r4c4
            )
        },
        (2, 2) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c3, mat.r1c4, 
                mat.r3c1, mat.r3c3, mat.r3c4, 
                mat.r4c1, mat.r4c3, mat.r4c4
            )
        },
        (2, 3) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c2, mat.r1c4, 
                mat.r3c1, mat.r3c2, mat.r3c4, 
                mat.r4c1, mat.r4c2, mat.r4c4
            )
        },
        (2, 4) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c2, mat.r1c3, 
                mat.r3c1, mat.r3c2, mat.r3c3, 
                mat.r4c1, mat.r4c2, mat.r4c3
            )
        },
        (3, 1) => {
            Mat3x3::new(
                mat.r1c2, mat.r1c3, mat.r1c4, 
                mat.r2c2, mat.r2c3, mat.r2c4, 
                mat.r4c2, mat.r4c3, mat.r4c4
            )
        },
        (3, 2) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c3, mat.r1c4, 
                mat.r2c1, mat.r2c3, mat.r2c4, 
                mat.r4c1, mat.r4c3, mat.r4c4
            )
        },
        (3, 3) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c2, mat.r1c4, 
                mat.r2c1, mat.r2c2, mat.r2c4, 
                mat.r4c1, mat.r4c2, mat.r4c4
            )
        },
        (3, 4) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c2, mat.r1c3, 
                mat.r2c1, mat.r2c2, mat.r2c3, 
                mat.r4c1, mat.r4c2, mat.r4c3
            )
        },
        (4, 1) => {
            Mat3x3::new(
                mat.r1c2, mat.r1c3, mat.r1c4, 
                mat.r2c2, mat.r2c3, mat.r2c4, 
                mat.r3c2, mat.r3c3, mat.r3c4
            )
        },
        (4, 2) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c3, mat.r1c4, 
                mat.r2c1, mat.r2c3, mat.r2c4, 
                mat.r3c1, mat.r3c3, mat.r3c4
            )
        },
        (4, 3) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c2, mat.r1c4, 
                mat.r2c1, mat.r2c2, mat.r2c4, 
                mat.r3c1, mat.r3c2, mat.r3c4
            )
        },
        (4, 4) => {
            Mat3x3::new(
                mat.r1c1, mat.r1c2, mat.r1c3, 
                mat.r2c1, mat.r2c2, mat.r2c3, 
                mat.r3c1, mat.r3c2, mat.r3c3
            )
        }
        _ => { panic!("out of range!") }
    }
}
