use std::cmp;
use std::ops;
use std::fmt;
use super::vec2::Vec2;

/// 2by2 matrix.
/// - row major
/// - pre-multiplication
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Mat2x2 {
    pub r1c1: f32, pub r1c2: f32,
    pub r2c1: f32, pub r2c2: f32 
}

impl Mat2x2 {
    /// matrix with all elements `0`.
    pub const ZERO: Self = Self::new_scalar(0.0);

    /// identity matrix.
    pub const IDENTITY: Self = Self::new_rows(
        Vec2::new_vector(1.0, 0.0), 
        Vec2::new_vector(0.0, 1.0)
    );

    /// create a matrix with the values of the given elements.
    #[inline]
    pub const fn new(r1c1: f32, r1c2: f32, r2c1: f32, r2c2: f32) -> Self {
        Self { r1c1, r1c2, r2c1, r2c2 }
    }

    /// create a matrix with the given scalar value.
    #[inline]
    pub const fn new_scalar(scalar: f32) -> Self {
        Self { r1c1: scalar, r1c2: scalar, r2c1: scalar, r2c2: scalar }
    }

    /// create a matrix with given row-major vectors.
    #[inline]
    pub const fn new_rows(row1: Vec2, row2: Vec2) -> Self {
        Self { r1c1: row1.x, r1c2: row1.y, r2c1: row2.x, r2c2: row2.y }
    }

    #[inline]
    pub fn add_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 + rhs, r1c2: self.r1c2 + rhs,
            r2c1: self.r2c1 + rhs, r2c2: self.r2c2 + rhs 
        }
    }

    #[inline]
    pub fn add_assign_scalar(&mut self, rhs: f32) {
        *self = self.add_scalar(rhs)
    }

    #[inline]
    pub fn add_matrix2x2(self, rhs: Self) -> Self {
        Self {
            r1c1: self.r1c1 + rhs.r1c1, r1c2: self.r1c2 + rhs.r1c2,
            r2c1: self.r2c1 + rhs.r2c1, r2c2: self.r2c2 + rhs.r2c2 
        }
    }

    #[inline]
    pub fn add_assign_matrix2x2(&mut self, rhs: Self) {
        *self = self.add_matrix2x2(rhs)
    }

    #[inline]
    pub fn sub_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 - rhs, r1c2: self.r1c2 - rhs,
            r2c1: self.r2c1 - rhs, r2c2: self.r2c2 - rhs 
        }
    }

    #[inline]
    pub fn sub_assign_scalar(&mut self, rhs: f32) {
        *self = self.sub_scalar(rhs)
    }

    #[inline]
    pub fn sub_matrix2x2(self, rhs: Self) -> Self {
        Self {
            r1c1: self.r1c1 - rhs.r1c1, r1c2: self.r1c2 - rhs.r1c2,
            r2c1: self.r2c1 - rhs.r2c1, r2c2: self.r2c2 - rhs.r2c2 
        }
    }

    #[inline]
    pub fn sub_assign_matrix2x2(&mut self, rhs: Self) {
        *self = self.sub_matrix2x2(rhs)
    }

    #[inline]
    pub fn mul_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 * rhs, r1c2: self.r1c2 * rhs,
            r2c1: self.r2c1 * rhs, r2c2: self.r2c2 * rhs 
        }
    }

    #[inline]
    pub fn mul_assign_scalar(&mut self, rhs: f32) {
        *self = self.mul_scalar(rhs)
    }

    #[inline]
    pub fn mul_matrix2x2(self, rhs: Self) -> Self {
        Self {
            r1c1: self.r1c1 * rhs.r1c1 + self.r1c2 * rhs.r2c1,
            r2c1: self.r2c1 * rhs.r1c1 + self.r2c2 * rhs.r2c1,

            r1c2: self.r1c1 * rhs.r1c2 + self.r1c2 * rhs.r2c2,
            r2c2: self.r2c1 * rhs.r1c2 + self.r2c2 * rhs.r2c2 
        }
    }

    #[inline]
    pub fn mul_assign_matrix2x2(&mut self, rhs: Self) {
        *self = self.mul_matrix2x2(rhs)
    }

    #[inline]
    pub fn div_scalar(self, rhs: f32) -> Self {
        Self {
            r1c1: self.r1c1 / rhs, r1c2: self.r1c2 / rhs,
            r2c1: self.r2c1 / rhs, r2c2: self.r2c2 / rhs 
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
            r1c1: self.r1c1, r1c2: self.r2c1,
            r2c1: self.r1c2, r2c2: self.r2c2 
        }
    }

    /// return a determinant of the matrix.
    #[inline]
    pub fn determinant(&self) -> f32 {
        self.r1c1 * self.r2c2 - self.r2c1 * self.r1c2
    }

    /// return inverse matrix.
    #[inline]
    pub fn inverse(&self) -> Self {
        let mt = self.transpose();
        let det = self.determinant();

        let cof_r1c1 = 1.0 * minor_matrix(&mt, 1, 1);
        let cof_r1c2 = -1.0 * minor_matrix(&mt, 1, 2);

        let cof_r2c1 = -1.0 * minor_matrix(&mt, 2, 1);
        let cof_r2c2 = 1.0 * minor_matrix(&mt, 2, 2);

        Self {
            r1c1: cof_r1c1 / det, r1c2: cof_r1c2 / det,
            r2c1: cof_r2c1 / det, r2c2: cof_r2c2 / det 
        }
    }

    /// return `None` if matrix cannot be create inverse matrix.
    #[inline]
    pub fn try_inverse(&self) -> Option<Self> {
        let mt = self.transpose();
        let det = self.determinant();

        if det.abs() > f32::EPSILON {
            let cof_r1c1 = 1.0 * minor_matrix(&mt, 1, 1);
            let cof_r1c2 = -1.0 * minor_matrix(&mt, 1, 2);

            let cof_r2c1 = -1.0 * minor_matrix(&mt, 2, 1);
            let cof_r2c2 = 1.0 * minor_matrix(&mt, 2, 2);

            return Some(Self {
                r1c1: cof_r1c1 / det, r1c2: cof_r1c2 / det,
                r2c1: cof_r2c1 / det, r2c2: cof_r2c2 / det 
            });
        }
        return None;
    }

    /// return `true` if any element of the matrix has the value of infinity.
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.r1c1.is_infinite() | self.r1c2.is_infinite()
        | self.r2c1.is_infinite() | self.r2c2.is_infinite()
    }

    /// return `true` if all elements of the matrix are neither infinite nor NaN.
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.r1c1.is_finite() & self.r1c2.is_finite()
        & self.r2c1.is_finite() & self.r2c2.is_finite()
    }

    /// return `true` if any element of the matrix has the value of NaN.
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.r1c1.is_nan() | self.r1c2.is_nan()
        | self.r2c1.is_nan() | self.r2c2.is_nan()
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
            r1c1: self.r1c1.min(other.r1c1), r1c2: self.r1c2.min(other.r1c2),
            r2c1: self.r2c1.min(other.r2c1), r2c2: self.r2c2.min(other.r2c2)
        }
    }

    /// return the greater of the elements of two matrices.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self { 
            r1c1: self.r1c1.max(other.r1c1), r1c2: self.r1c2.max(other.r1c2),
            r2c1: self.r2c1.max(other.r2c1), r2c2: self.r2c2.max(other.r2c2)
        }
    }

    /// round up the decimal places of the elements of a matrix.
    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            r1c1: self.r1c1.ceil(), r1c2: self.r1c2.ceil(),
            r2c1: self.r2c1.ceil(), r2c2: self.r2c2.ceil()
        }
    }

    /// round down the decimal places of the elements of a matrix.
    #[inline]
    pub fn floor(self) -> Self {
        Self {
            r1c1: self.r1c1.floor(), r1c2: self.r1c2.floor(),
            r2c1: self.r2c1.floor(), r2c2: self.r2c2.floor()
        }
    }

    /// round the decimal places of the elements of a matrix.
    #[inline]
    pub fn round(self) -> Self {
        Self {
            r1c1: self.r1c1.round(), r1c2: self.r1c2.round(),
            r2c1: self.r2c1.round(), r2c2: self.r2c2.round()
        }
    }
}


impl ops::Add<Mat2x2> for f32 {
    type Output = Mat2x2;
    #[inline]
    fn add(self, rhs: Mat2x2) -> Self::Output {
        Mat2x2 {
            r1c1: self + rhs.r1c1, r1c2: self + rhs.r1c2,
            r2c1: self + rhs.r2c1, r2c2: self + rhs.r2c2 
        }
    }
}

impl ops::Add<f32> for Mat2x2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        self.add_scalar(rhs)
    }
}

impl ops::AddAssign<f32> for Mat2x2 {
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.add_assign_scalar(rhs)
    }
}

impl ops::Add<Self> for Mat2x2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.add_matrix2x2(rhs)
    }
}

impl ops::AddAssign<Self> for Mat2x2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign_matrix2x2(rhs)
    }
}

impl ops::Sub<Mat2x2> for f32 {
    type Output = Mat2x2;
    #[inline]
    fn sub(self, rhs: Mat2x2) -> Self::Output {
        Mat2x2 {
            r1c1: self - rhs.r1c1, r1c2: self - rhs.r1c2,
            r2c1: self - rhs.r2c1, r2c2: self - rhs.r2c2
        }
    }
}

impl ops::Sub<f32> for Mat2x2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        self.sub_scalar(rhs)
    }
}

impl ops::SubAssign<f32> for Mat2x2 {
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.sub_assign_scalar(rhs)
    }
}

impl ops::Sub<Self> for Mat2x2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_matrix2x2(rhs)
    }
}

impl ops::SubAssign<Self> for Mat2x2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign_matrix2x2(rhs)
    }
}

impl ops::Neg for Mat2x2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            r1c1: self.r1c1.neg(), r1c2: self.r1c2.neg(),
            r2c1: self.r2c1.neg(), r2c2: self.r2c2.neg() 
        }
    }
}

impl ops::Mul<Mat2x2> for f32 {
    type Output = Mat2x2;
    #[inline]
    fn mul(self, rhs: Mat2x2) -> Self::Output {
        Mat2x2 {
            r1c1: self * rhs.r1c1, r1c2: self * rhs.r1c2,
            r2c1: self * rhs.r2c1, r2c2: self * rhs.r2c2 
        }
    }
}

impl ops::Mul<f32> for Mat2x2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        self.mul_scalar(rhs)
    }
}

impl ops::MulAssign<f32> for Mat2x2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.mul_assign_scalar(rhs)
    }
}

impl ops::Mul<Self> for Mat2x2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_matrix2x2(rhs)
    }
}

impl ops::MulAssign<Self> for Mat2x2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.mul_assign_matrix2x2(rhs)
    }
}

impl ops::Div<Mat2x2> for f32 {
    type Output = Mat2x2;
    #[inline]
    fn div(self, rhs: Mat2x2) -> Self::Output {
        Mat2x2 {
            r1c1: self / rhs.r1c1, r1c2: self / rhs.r1c2,
            r2c1: self / rhs.r2c1, r2c2: self / rhs.r2c2 
        }
    }
}

impl ops::Div<f32> for Mat2x2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        self.div_scalar(rhs)
    }
}

impl ops::DivAssign<f32> for Mat2x2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.div_assign_scalar(rhs)
    }
}

impl cmp::PartialEq<Self> for Mat2x2 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

impl AsRef<[f32; 4]> for Mat2x2 {
    #[inline]
    fn as_ref(&self) -> &[f32; 4] {
        unsafe { &*(self as *const Self as *const [f32; 4]) }
    }
}

impl AsMut<[f32; 4]> for Mat2x2 {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 4] {
        unsafe { &mut *(self as *mut Self as *mut [f32; 4]) }
    }
}

impl fmt::Display for Mat2x2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[({}, {}), ({}, {})]", self.r1c1, self.r1c2, self.r2c1, self.r2c2)
    }
}

#[inline]
fn minor_matrix(mat: &Mat2x2, row: usize, col: usize) -> f32 {
    debug_assert!(0 < row && row <= 2, "row out of range!");
    debug_assert!(0 < col && col <= 2, "column out of range!");
    match (row, col) {
        (1, 1) => { mat.r2c2 },
        (1, 2) => { mat.r2c1 },
        (2, 1) => { mat.r1c2 },
        (2, 2) => { mat.r1c1 },
        _ => { panic!("out of range!") }
    }
}
