use std::ops;
use std::fmt;
use std::cmp;
use bytemuck::{Zeroable, Pod};
use super::mat2::Mat2x2;
use super::quat::Quat;
use super::vec3::Vec3;

/// 3by3 matrix.
/// - row major
/// - pre-multiplicaiton
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Zeroable, Pod)]
pub struct Mat3x3 {
    pub r1c1: f32, pub r1c2: f32, pub r1c3: f32,
    pub r2c1: f32, pub r2c2: f32, pub r2c3: f32,
    pub r3c1: f32, pub r3c2: f32, pub r3c3: f32
}

impl Mat3x3 {
    /// matrix with all elements `0`.
    pub const ZERO: Self = Self::new_scalar(0.0);

    /// identity matrix.
    pub const IDENTITY: Self = Self::new_rows(
        Vec3::new_vector(1.0, 0.0, 0.0), 
        Vec3::new_vector(0.0, 1.0, 0.0), 
        Vec3::new_vector(0.0, 0.0, 1.0)
    );

    /// create a matrix with the values of the given elements.
    #[inline]
    pub const fn new(
        r1c1: f32, r1c2: f32, r1c3: f32,
        r2c1: f32, r2c2: f32, r2c3: f32,
        r3c1: f32, r3c2: f32, r3c3: f32
    ) -> Self {
        Self { 
            r1c1, r1c2, r1c3, 
            r2c1, r2c2, r2c3, 
            r3c1, r3c2, r3c3 
        }
    }

    /// create a matrix with the given scalar value.
    #[inline]
    pub const fn new_scalar(scalar: f32) -> Self {
        Self { 
            r1c1: scalar, r1c2: scalar, r1c3: scalar, 
            r2c1: scalar, r2c2: scalar, r2c3: scalar, 
            r3c1: scalar, r3c2: scalar, r3c3: scalar 
        }
    }

    /// create a matrix with given row-major vectors.
    #[inline]
    pub const fn new_rows(row1: Vec3, row2: Vec3, row3: Vec3) -> Self {
        Self {
            r1c1: row1.x, r1c2: row1.y, r1c3: row1.z,
            r2c1: row2.x, r2c2: row2.y, r2c3: row2.z,
            r3c1: row3.x, r3c2: row3.y, r3c3: row3.z,
        }
    }

    /// create a matrix with given quaternion.
    #[inline]
    pub fn from_quat(quat: Quat) -> Self {
        Self {
            r1c1: 1.0 - 2.0 * quat.y * quat.y - 2.0 * quat.z * quat.z,
            r1c2: 2.0 * quat.x * quat.y + 2.0 * quat.z * quat.w,
            r1c3: 2.0 * quat.x * quat.z - 2.0 * quat.y * quat.w,
            
            r2c1: 2.0 * quat.x * quat.y - 2.0 * quat.z * quat.w,
            r2c2: 1.0 - 2.0 * quat.x * quat.x - 2.0 * quat.z * quat.z,
            r2c3: 2.0 * quat.y * quat.z + 2.0 * quat.x * quat.w,

            r3c1: 2.0 * quat.x * quat.z + 2.0 * quat.y * quat.w,
            r3c2: 2.0 * quat.y * quat.z - 2.0 * quat.x * quat.w,
            r3c3: 1.0 - 2.0 * quat.x * quat.x - 2.0 * quat.y * quat.y,
        }
    }
    
    /// convert a matrix to an quaternion.
    #[inline]
    pub fn into_quat(self) -> Quat {
        Quat::from_matrix3x3(self)
    }

    #[inline]
    pub fn add_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 + rhs, r1c2: self.r1c2 + rhs, r1c3: self.r1c3 + rhs,
            r2c1: self.r2c1 + rhs, r2c2: self.r2c2 + rhs, r2c3: self.r2c3 + rhs,
            r3c1: self.r3c1 + rhs, r3c2: self.r3c2 + rhs, r3c3: self.r3c3 + rhs
        }
    }

    #[inline]
    pub fn add_assign_scalar(&mut self, rhs: f32) {
        *self = self.add_scalar(rhs)
    }

    #[inline]
    pub fn add_matrix3x3(self, rhs: Self) -> Self {
        Self {
            r1c1: self.r1c1 + rhs.r1c1, r1c2: self.r1c2 + rhs.r1c2, r1c3: self.r1c3 + rhs.r1c3,
            r2c1: self.r2c1 + rhs.r2c1, r2c2: self.r2c2 + rhs.r2c2, r2c3: self.r2c3 + rhs.r2c3,
            r3c1: self.r3c1 + rhs.r3c1, r3c2: self.r3c2 + rhs.r3c2, r3c3: self.r3c3 + rhs.r3c3
        }
    }

    #[inline]
    pub fn add_assign_matrix3x3(&mut self, rhs: Self) {
        *self = self.add_matrix3x3(rhs)
    }

    #[inline]
    pub fn sub_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 - rhs, r1c2: self.r1c2 - rhs, r1c3: self.r1c3 - rhs,
            r2c1: self.r2c1 - rhs, r2c2: self.r2c2 - rhs, r2c3: self.r2c3 - rhs,
            r3c1: self.r3c1 - rhs, r3c2: self.r3c2 - rhs, r3c3: self.r3c3 - rhs,
        }
    }

    #[inline]
    pub fn sub_assign_scalar(&mut self, rhs: f32) {
        *self = self.sub_scalar(rhs)
    }

    #[inline]
    pub fn sub_matrix3x3(self, rhs: Self) -> Self {
        Self {
            r1c1: self.r1c1 - rhs.r1c1, r1c2: self.r1c2 - rhs.r1c2, r1c3: self.r1c3 - rhs.r1c3,
            r2c1: self.r2c1 - rhs.r2c1, r2c2: self.r2c2 - rhs.r2c2, r2c3: self.r2c3 - rhs.r2c3,
            r3c1: self.r3c1 - rhs.r3c1, r3c2: self.r3c2 - rhs.r3c2, r3c3: self.r3c3 - rhs.r3c3
        }
    }

    #[inline]
    pub fn sub_assign_matrix3x3(&mut self, rhs: Self) {
        *self = self.sub_matrix3x3(rhs)
    }

    #[inline]
    pub fn mul_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 * rhs, r1c2: self.r1c2 * rhs, r1c3: self.r1c3 * rhs,
            r2c1: self.r2c1 * rhs, r2c2: self.r2c2 * rhs, r2c3: self.r2c3 * rhs,
            r3c1: self.r3c1 * rhs, r3c2: self.r3c2 * rhs, r3c3: self.r3c3 * rhs,
        }
    }

    #[inline]
    pub fn mul_assign_scalar(&mut self, rhs: f32) {
        *self = self.mul_scalar(rhs)
    }

    #[inline]
    pub fn mul_matrix3x3(self, rhs: Self) -> Self {
        Mat3x3 {
            r1c1: self.r1c1 * rhs.r1c1 + self.r1c2 * rhs.r2c1 + self.r1c3 * rhs.r3c1,
            r2c1: self.r2c1 * rhs.r1c1 + self.r2c2 * rhs.r2c1 + self.r2c3 * rhs.r3c1,
            r3c1: self.r3c1 * rhs.r1c1 + self.r3c2 * rhs.r2c1 + self.r3c3 * rhs.r3c1,

            r1c2: self.r1c1 * rhs.r1c2 + self.r1c2 * rhs.r2c2 + self.r1c3 * rhs.r3c2,
            r2c2: self.r2c1 * rhs.r1c2 + self.r2c2 * rhs.r2c2 + self.r2c3 * rhs.r3c2,
            r3c2: self.r3c1 * rhs.r1c2 + self.r3c2 * rhs.r2c2 + self.r3c3 * rhs.r3c2,

            r1c3: self.r1c1 * rhs.r1c3 + self.r1c2 * rhs.r2c3 + self.r1c3 * rhs.r3c3,
            r2c3: self.r2c1 * rhs.r1c3 + self.r2c2 * rhs.r2c3 + self.r2c3 * rhs.r3c3,
            r3c3: self.r3c1 * rhs.r1c3 + self.r3c2 * rhs.r2c3 + self.r3c3 * rhs.r3c3
        }
    }

    #[inline]
    pub fn mul_assign_matrix3x3(&mut self, rhs: Self) {
        *self = self.mul_matrix3x3(rhs)
    }

    #[inline]
    pub fn div_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 / rhs, r1c2: self.r1c2 / rhs, r1c3: self.r1c3 / rhs,
            r2c1: self.r2c1 / rhs, r2c2: self.r2c2 / rhs, r2c3: self.r2c3 / rhs,
            r3c1: self.r3c1 / rhs, r3c2: self.r3c2 / rhs, r3c3: self.r3c3 / rhs,
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
            r1c1: self.r1c1, r1c2: self.r2c1, r1c3: self.r3c1,
            r2c1: self.r1c2, r2c2: self.r2c2, r2c3: self.r3c2,
            r3c1: self.r1c3, r3c2: self.r2c3, r3c3: self.r3c3 
        }
    }

    /// return a determinant of the matrix.
    #[inline]
    pub fn determinant(&self) -> f32 {
        (self.r1c1 * self.r2c2 * self.r3c3 + self.r1c2 * self.r2c3 * self.r3c1 + self.r1c3 * self.r2c1 * self.r3c2)
        - (self.r1c1 * self.r2c3 * self.r3c2 + self.r1c2 * self.r2c1 * self.r3c3 + self.r1c3 * self.r2c2 * self.r3c1)
    }

    /// return inverse matrix.
    #[inline]
    pub fn inverse(&self) -> Self {
        let mt = self.transpose();
        let det = self.determinant();

        let cof_r1c1 = 1.0 * minor_matrix(&mt, 1, 1).determinant();
        let cof_r1c2 = -1.0 * minor_matrix(&mt, 1, 2).determinant();
        let cof_r1c3 = 1.0 * minor_matrix(&mt, 1, 3).determinant();
        
        let cof_r2c1 = -1.0 * minor_matrix(&mt, 2, 1).determinant();
        let cof_r2c2 = 1.0 * minor_matrix(&mt, 2, 2).determinant();
        let cof_r2c3 = -1.0 * minor_matrix(&mt, 2, 3).determinant();

        let cof_r3c1 = 1.0 * minor_matrix(&mt, 3, 1).determinant();
        let cof_r3c2 = -1.0 * minor_matrix(&mt, 3, 2).determinant();
        let cof_r3c3 = 1.0 * minor_matrix(&mt, 3, 3).determinant();

        Self { 
            r1c1: cof_r1c1 / det, r1c2: cof_r1c2 / det, r1c3: cof_r1c3 / det,
            r2c1: cof_r2c1 / det, r2c2: cof_r2c2 / det, r2c3: cof_r2c3 / det,
            r3c1: cof_r3c1 / det, r3c2: cof_r3c2 / det, r3c3: cof_r3c3 / det 
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
            
            let cof_r2c1 = -1.0 * minor_matrix(&mt, 2, 1).determinant();
            let cof_r2c2 = 1.0 * minor_matrix(&mt, 2, 2).determinant();
            let cof_r2c3 = -1.0 * minor_matrix(&mt, 2, 3).determinant();

            let cof_r3c1 = 1.0 * minor_matrix(&mt, 3, 1).determinant();
            let cof_r3c2 = -1.0 * minor_matrix(&mt, 3, 2).determinant();
            let cof_r3c3 = 1.0 * minor_matrix(&mt, 3, 3).determinant();

            return Some(Self { 
                r1c1: cof_r1c1 / det, r1c2: cof_r1c2 / det, r1c3: cof_r1c3 / det,
                r2c1: cof_r2c1 / det, r2c2: cof_r2c2 / det, r2c3: cof_r2c3 / det,
                r3c1: cof_r3c1 / det, r3c2: cof_r3c2 / det, r3c3: cof_r3c3 / det 
            });
        }
        return None;
    }

    /// return `true` if any element of the matrix has the value of infinity.
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.r1c1.is_infinite() | self.r1c2.is_infinite() | self.r1c3.is_infinite()
        | self.r2c1.is_infinite() | self.r2c2.is_infinite() | self.r2c3.is_infinite()
        | self.r3c1.is_infinite() | self.r3c2.is_infinite() | self.r3c3.is_infinite()
    }

    /// return `true` if all elements of the matrix are neither infinite nor NaN.
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.r1c1.is_finite() & self.r1c2.is_finite() & self.r1c3.is_finite()
        & self.r2c1.is_finite() & self.r2c2.is_finite() & self.r2c3.is_finite()
        & self.r3c1.is_finite() & self.r3c2.is_finite() & self.r3c3.is_finite()
    }

    /// return `true` if any element of the matrix has the value of NaN.
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.r1c1.is_nan() | self.r1c2.is_nan() | self.r1c3.is_nan()
        | self.r2c1.is_nan() | self.r2c2.is_nan() | self.r2c3.is_nan()
        | self.r3c1.is_nan() | self.r3c2.is_nan() | self.r3c3.is_nan()
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
            r1c1: self.r1c1.min(other.r1c1), r1c2: self.r1c2.min(other.r1c2), r1c3: self.r1c3.min(other.r1c3),
            r2c1: self.r2c1.min(other.r2c1), r2c2: self.r2c2.min(other.r2c2), r2c3: self.r2c3.min(other.r2c3),
            r3c1: self.r3c1.min(other.r3c1), r3c2: self.r3c2.min(other.r3c2), r3c3: self.r3c3.min(other.r3c3),
        }
    }

    /// return the greater of the elements of two matrices.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self { 
            r1c1: self.r1c1.max(other.r1c1), r1c2: self.r1c2.max(other.r1c2), r1c3: self.r1c3.max(other.r1c3),
            r2c1: self.r2c1.max(other.r2c1), r2c2: self.r2c2.max(other.r2c2), r2c3: self.r2c3.max(other.r2c3),
            r3c1: self.r3c1.max(other.r3c1), r3c2: self.r3c2.max(other.r3c2), r3c3: self.r3c3.max(other.r3c3),
        }
    }

    /// round up the decimal places of the elements of a matrix.
    #[inline]
    pub fn ceil(self) -> Self {
        Self { 
            r1c1: self.r1c1.ceil(), r1c2: self.r1c2.ceil(), r1c3: self.r1c3.ceil(),
            r2c1: self.r2c1.ceil(), r2c2: self.r2c2.ceil(), r2c3: self.r2c3.ceil(),
            r3c1: self.r3c1.ceil(), r3c2: self.r3c2.ceil(), r3c3: self.r3c3.ceil(),
        }
    }

    /// round down the decimal places of the elements of a matrix.
    #[inline]
    pub fn floor(self) -> Self {
        Self { 
            r1c1: self.r1c1.floor(), r1c2: self.r1c2.floor(), r1c3: self.r1c3.floor(),
            r2c1: self.r2c1.floor(), r2c2: self.r2c2.floor(), r2c3: self.r2c3.floor(),
            r3c1: self.r3c1.floor(), r3c2: self.r3c2.floor(), r3c3: self.r3c3.floor(),
        }
    }

    /// round the decimal places of the elements of a matrix.
    #[inline]
    pub fn round(self) -> Self {
        Self { 
            r1c1: self.r1c1.round(), r1c2: self.r1c2.round(), r1c3: self.r1c3.round(),
            r2c1: self.r2c1.round(), r2c2: self.r2c2.round(), r2c3: self.r2c3.round(),
            r3c1: self.r3c1.round(), r3c2: self.r3c2.round(), r3c3: self.r3c3.round(),
        }
    }
}


impl ops::Add<Mat3x3> for f32 {
    type Output = Mat3x3;
    #[inline]
    fn add(self, rhs: Mat3x3) -> Self::Output {
        Mat3x3 {
            r1c1: self + rhs.r1c1, r1c2: self + rhs.r1c2, r1c3: self + rhs.r1c3,
            r2c1: self + rhs.r2c1, r2c2: self + rhs.r2c2, r2c3: self + rhs.r2c3,
            r3c1: self + rhs.r3c1, r3c2: self + rhs.r3c2, r3c3: self + rhs.r3c3 
        }
    }
}

impl ops::Add<f32> for Mat3x3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        self.add_scalar(rhs)
    }
}

impl ops::AddAssign<f32> for Mat3x3 {
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.add_assign_scalar(rhs)
    }
}

impl ops::Add<Self> for Mat3x3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.add_matrix3x3(rhs)
    }
}

impl ops::AddAssign<Self> for Mat3x3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign_matrix3x3(rhs)
    }
}

impl ops::Sub<Mat3x3> for f32 {
    type Output = Mat3x3;
    #[inline]
    fn sub(self, rhs: Mat3x3) -> Self::Output {
        Mat3x3 {
            r1c1: self - rhs.r1c1, r1c2: self - rhs.r1c2, r1c3: self - rhs.r1c3,
            r2c1: self - rhs.r2c1, r2c2: self - rhs.r2c2, r2c3: self - rhs.r2c3,
            r3c1: self - rhs.r3c1, r3c2: self - rhs.r3c2, r3c3: self - rhs.r3c3 
        }
    }
}

impl ops::Sub<f32> for Mat3x3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        self.sub_scalar(rhs)
    }
}

impl ops::SubAssign<f32> for Mat3x3 {
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.sub_assign_scalar(rhs)
    }
}

impl ops::Sub<Self> for Mat3x3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_matrix3x3(rhs)
    }
}

impl ops::SubAssign<Self> for Mat3x3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign_matrix3x3(rhs)
    }
}

impl ops::Neg for Mat3x3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            r1c1: self.r1c1.neg(), r1c2: self.r1c2.neg(), r1c3: self.r1c3.neg(),
            r2c1: self.r2c1.neg(), r2c2: self.r2c2.neg(), r2c3: self.r2c3.neg(),
            r3c1: self.r3c1.neg(), r3c2: self.r3c2.neg(), r3c3: self.r3c3.neg()
        }
    }
}

impl ops::Mul<Mat3x3> for f32 {
    type Output = Mat3x3;
    #[inline]
    fn mul(self, rhs: Mat3x3) -> Self::Output {
        Mat3x3 {
            r1c1: self * rhs.r1c1, r1c2: self * rhs.r1c2, r1c3: self * rhs.r1c3,
            r2c1: self * rhs.r2c1, r2c2: self * rhs.r2c2, r2c3: self * rhs.r2c3,
            r3c1: self * rhs.r3c1, r3c2: self * rhs.r3c2, r3c3: self * rhs.r3c3 
        }
    }
}

impl ops::Mul<f32> for Mat3x3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        self.mul_scalar(rhs)
    }
}

impl ops::MulAssign<f32> for Mat3x3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.mul_assign_scalar(rhs)
    }
}

impl ops::Mul<Self> for Mat3x3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_matrix3x3(rhs)
    }
}

impl ops::MulAssign<Self> for Mat3x3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.mul_assign_matrix3x3(rhs)
    }
}

impl ops::Div<Mat3x3> for f32 {
    type Output = Mat3x3;
    #[inline]
    fn div(self, rhs: Mat3x3) -> Self::Output {
        Mat3x3 {
            r1c1: self / rhs.r1c1, r1c2: self / rhs.r1c2, r1c3: self / rhs.r1c3,
            r2c1: self / rhs.r2c1, r2c2: self / rhs.r2c2, r2c3: self / rhs.r2c3,
            r3c1: self / rhs.r3c1, r3c2: self / rhs.r3c2, r3c3: self / rhs.r3c3 
        }
    }
}

impl ops::Div<f32> for Mat3x3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        self.div_scalar(rhs)
    }
}

impl ops::DivAssign<f32> for Mat3x3 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.div_assign_scalar(rhs)
    }
}

impl cmp::PartialEq<Self> for Mat3x3 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

impl AsRef<[f32; 9]> for Mat3x3 {
    #[inline]
    fn as_ref(&self) -> &[f32; 9] {
        unsafe { &*(self as *const Self as *const [f32; 9]) }
    }
}

impl AsMut<[f32; 9]> for Mat3x3 {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 9] {
        unsafe { &mut *(self as *mut Self as *mut [f32; 9]) }
    }
}

impl fmt::Display for Mat3x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, 
            "[({}, {}, {}), ({}, {}, {}), ({}, {}, {})]", 
            self.r1c1, self.r1c2, self.r1c3,
            self.r2c1, self.r2c2, self.r2c3,
            self.r3c1, self.r3c2, self.r3c3
        )
    }
}

#[inline]
fn minor_matrix(mat: &Mat3x3, row: usize, col: usize) -> Mat2x2 {
    debug_assert!(0 < row && row <= 3, "row out of range!");
    debug_assert!(0 < col && col <= 3, "column out of range!");
    match (row, col) {
        (1, 1) => { 
            Mat2x2::new(mat.r2c2, mat.r2c3, mat.r3c2, mat.r3c3)
        },
        (1, 2) => {
            Mat2x2::new(mat.r2c1, mat.r2c3, mat.r3c1, mat.r3c3)
        },
        (1, 3) => {
            Mat2x2::new(mat.r2c1, mat.r2c2, mat.r3c1, mat.r3c2)
        },
        (2, 1) => {
            Mat2x2::new(mat.r1c2, mat.r1c3, mat.r3c2, mat.r3c3)
        },
        (2, 2) => {
            Mat2x2::new(mat.r1c1, mat.r1c3, mat.r3c1, mat.r3c3)
        },
        (2, 3) => {
            Mat2x2::new(mat.r1c1, mat.r1c2, mat.r3c1, mat.r3c2)
        },
        (3, 1) => {
            Mat2x2::new(mat.r1c2, mat.r1c3, mat.r2c2, mat.r2c3)
        },
        (3, 2) => {
            Mat2x2::new(mat.r1c1, mat.r1c3, mat.r2c1, mat.r2c3)
        },
        (3, 3) => {
            Mat2x2::new(mat.r1c1, mat.r1c2, mat.r2c1, mat.r2c2)
        },
        _ => { panic!("out of range!") }
    }
}
