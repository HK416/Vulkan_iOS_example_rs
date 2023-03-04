use std::fmt;
use std::ops;
use std::cmp;
use super::mat2::Mat2x2;

/// 2-dimensional vector.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32 
}

impl Vec2 {
    /// vector with all elements `0`.
    pub const ZERO: Self = Self::new_scalar(0.0);

    /// vector with all elements `1`.
    pub const ONE: Self = Self::new_scalar(1.0);

    /// A vector in which only the elements on the x-axis are `1` and the rest are `0`.
    pub const X: Self = Self::new_vector(1.0, 0.0);
    
    /// A vector in which only the elements on the y-axis are `1` and the rest are `0`.
    pub const Y: Self = Self::new_vector(0.0, 1.0);

    /// vector with all elements `f32::MIN`.
    pub const MIN: Self = Self::new_scalar(f32::MIN);
    
    /// vector with all elements `f32::MAX`.
    pub const MAX: Self = Self::new_scalar(f32::MAX);

    /// vector with all elements `f32::NAN`.
    pub const NAN: Self = Self::new_scalar(f32::NAN);

    /// vector with all elements `f32::INFINITY`.
    pub const INFINITY: Self = Self::new_scalar(f32::INFINITY);

    /// create a vector with the given scalar value.
    #[inline]
    pub const fn new_scalar(scalar: f32) -> Self {
        Self { x: scalar, y: scalar }
    }

    /// create a vector with the values of the given elements.
    #[inline]
    pub const fn new_vector(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// create a vector with the values of the given array.
    #[inline]
    pub const fn from_array(arr: [f32; 2]) -> Self {
        Self { x: arr[0], y: arr[1] }
    }

    /// convert a vector to an array.
    #[inline]
    pub const fn into_array(self) -> [f32; 2] {
        [self.x, self.y]
    }

    /// create a vector with the values of the given tuple.
    #[inline]
    pub const fn from_tuple(tup: (f32, f32)) -> Self {
        Self { x: tup.0, y: tup.1 }
    }

    /// convert a vector to an tuple.
    #[inline]
    pub const fn into_tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }

    #[inline]
    pub fn add_scalar(self, rhs: f32) -> Self {
        Self {
            x: self.x + rhs,
            y: self.y + rhs 
        }
    }

    #[inline]
    pub fn add_assign_scalar(&mut self, rhs: f32) {
        *self = self.add_scalar(rhs)
    }

    #[inline]
    pub fn add_vector2(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y 
        }
    }

    #[inline]
    pub fn add_assign_vector2(&mut self, rhs: Self) {
        *self = self.add_vector2(rhs)
    }

    #[inline]
    pub fn sub_scalar(self, rhs: f32) -> Self {
        Self {
            x: self.x - rhs,
            y: self.y - rhs 
        }
    }

    #[inline]
    pub fn sub_assign_scalar(&mut self, rhs: f32) {
        *self = self.sub_scalar(rhs)
    }

    #[inline]
    pub fn sub_vector2(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y 
        }
    }

    #[inline]
    pub fn sub_assign_vector2(&mut self, rhs: Self) {
        *self = self.sub_vector2(rhs)
    }

    #[inline]
    pub fn mul_scalar(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs 
        }
    }

    #[inline]
    pub fn mul_assign_scalar(&mut self, rhs: f32) {
        *self = self.mul_scalar(rhs)
    }

    #[inline]
    pub fn mul_vector2(self, rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y 
        }
    }

    #[inline]
    pub fn mul_assign_vector2(&mut self, rhs: Self) {
        *self = self.mul_vector2(rhs)
    }

    #[inline]
    pub fn mul_matrix2x2(self, rhs: Mat2x2) -> Self {
        Self {
            x: self.x * rhs.r1c1 + self.y * rhs.r2c1,
            y: self.x * rhs.r1c2 + self.y * rhs.r2c2 
        }
    }

    #[inline]
    pub fn mul_assign_matrix2x2(&mut self, rhs: Mat2x2) {
        *self = self.mul_matrix2x2(rhs)
    }

    #[inline]
    pub fn div_scalar(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs 
        }
    }

    #[inline]
    pub fn div_assign_scalar(&mut self, rhs: f32) {
        *self = self.div_scalar(rhs)
    }

    #[inline]
    pub fn div_vector2(self, rhs: Self) -> Self {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y 
        }
    }

    #[inline]
    pub fn div_assign_vector2(&mut self, rhs: Self) {
        *self = self.div_vector2(rhs)
    }

    /// dot product of two vectors.
    #[inline]
    pub fn dot(&self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    /// the length of the vector.
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// the square of the length of the vector.
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// return normalized vector.
    #[inline]
    pub fn normalize(&self) -> Self {
        self.div_scalar(self.length())
    }

    /// return `true` if it is a normalized vector.
    #[inline]
    pub fn is_normalized(&self) -> bool {
        (self.length_squared() - 1.0).abs() <= f32::EPSILON
    }

    /// return `None` if vector cannot be normalized.
    #[inline]
    pub fn try_normalized(&self) -> Option<Self> {
        let length = self.length();
        if length > f32::EPSILON {
            return Some(self.div_scalar(length));
        }
        return None;
        
    }

    /// return `true` if any element of the vector has the value of infinity.
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.x.is_infinite() | self.y.is_infinite()
    }

    /// return `true` if all elements of the vector are neither infinite nor NaN.
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() & self.y.is_finite()
    }

    /// return `true` if any element of the vector has the value of NaN.
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.x.is_nan() | self.y.is_nan()
    }

    /// return `true` if the two vectors are equal.
    #[inline]
    pub fn equal(&self, other: &Self) -> bool {
        let mut flag = true;
        for &num in (*self - *other).as_ref().iter() {
            flag &= num.abs() <= f32::EPSILON
        }
        return flag;
    }

    /// return the smaller of the elements of two vectors.
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self { 
            x: self.x.min(other.x),
            y: self.y.min(other.y)
        }
    }

    /// return the greater of the elements of two vectors.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y)
        }
    }

    /// round up the decimal places of the elements of a vector.
    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil() 
        }
    }

    /// round down the decimal places of the elements of a vector.
    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor() 
        }
    }

    /// round the decimal places of the elements of a vector.
    #[inline]
    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round() 
        }
    }
}


impl ops::Add<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self + rhs.x,
            y: self + rhs.y 
        }
    }
}

impl ops::Add<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        self.add_scalar(rhs)
    }
}

impl ops::AddAssign<f32> for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.add_assign_scalar(rhs)
    }
}

impl ops::Add<Self> for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.add_vector2(rhs)
    }
}

impl ops::AddAssign<Self> for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign_vector2(rhs)
    }
}

impl ops::Sub<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self - rhs.x,
            y: self - rhs.y 
        }
    }
}

impl ops::Sub<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        self.sub_scalar(rhs)
    }
}

impl ops::SubAssign<f32> for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.sub_assign_scalar(rhs)
    }
}

impl ops::Sub<Self> for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_vector2(rhs)
    }
}

impl ops::SubAssign<Self> for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign_vector2(rhs)
    }
}

impl ops::Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: self.x.neg(),
            y: self.y.neg() 
        }
    }
}

impl ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y 
        }
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        self.mul_scalar(rhs)
    }
}

impl ops::MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.mul_assign_scalar(rhs)
    }
}

impl ops::Mul<Self> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_vector2(rhs)
    }
}

impl ops::MulAssign<Self> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.mul_assign_vector2(rhs)
    }
}

impl ops::Mul<Mat2x2> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Mat2x2) -> Self::Output {
        self.mul_matrix2x2(rhs)
    }
}

impl ops::MulAssign<Mat2x2> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Mat2x2) {
        self.mul_assign_matrix2x2(rhs)
    }
}

impl ops::Div<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn div(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self / rhs.x,
            y: self / rhs.y 
        }
    }
}

impl ops::Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        self.div_scalar(rhs)
    }
}

impl ops::DivAssign<f32> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.div_assign_scalar(rhs)
    }
}

impl ops::Div<Self> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.div_vector2(rhs)
    }
}

impl ops::DivAssign<Self> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.div_assign_vector2(rhs)
    }
}

impl ops::Index<usize> for Vec2 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("index out of range.")
        }
    }
}

impl ops::IndexMut<usize> for Vec2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("index out of range.")
        }
    }
}

impl cmp::PartialEq for Vec2 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

impl From<[f32; 2]> for Vec2 {
    #[inline]
    fn from(arr: [f32; 2]) -> Self {
        Self::from_array(arr)
    }
}

impl Into<[f32; 2]> for Vec2 {
    #[inline]
    fn into(self) -> [f32; 2] {
        self.into_array()
    }
}

impl From<(f32, f32)> for Vec2 {
    #[inline]
    fn from(tup: (f32, f32)) -> Self {
        Self::from_tuple(tup)
    }
}

impl Into<(f32, f32)> for Vec2 {
    #[inline]
    fn into(self) -> (f32, f32) {
        self.into_tuple()
    }
}

impl AsRef<[f32; 2]> for Vec2 {
    #[inline]
    fn as_ref(&self) -> &[f32; 2] {
        unsafe { &*(self as *const Self as *const [f32; 2]) }
    }
}

impl AsMut<[f32; 2]> for Vec2 {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 2] {
        unsafe { &mut *(self as *mut Self as *mut [f32; 2]) }
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
