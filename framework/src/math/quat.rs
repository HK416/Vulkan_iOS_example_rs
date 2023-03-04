use std::ops;
use std::fmt;
use std::cmp;
use super::mat3::Mat3x3;
use super::mat4::Mat4x4;
use super::vec3::Vec3;
use super::vec4::Vec4;

/// quaternion.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32 
}

impl Quat {
    /// quaternion with all elements `0`.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// identity quaternion.
    pub const IDENTITY: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// create a quaternion with the values of the given elements.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// create a quaternion with the given vector.
    #[inline]
    pub const fn from_vector4(vec: Vec4) -> Self {
        Self { x: vec.x, y: vec.y, z: vec.z, w: vec.w }
    }

    /// convert a quaternion to an vector.
    #[inline]
    pub const fn into_vector4(self) -> Vec4 {
        Vec4 { x: self.x, y: self.y, z: self.z, w: self.w }
    }

    /// create a quaternion with the values of the given array.
    #[inline]
    pub const fn from_array(arr: [f32; 4]) -> Self {
        Self { x: arr[0], y: arr[1], z: arr[2], w: arr[3] }
    }

    /// convert a quaternion to an array.
    #[inline]
    pub const fn into_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    /// create a quaternion with the values of the given tuple.
    #[inline]
    pub const fn from_tuple(tup: (f32, f32, f32, f32)) -> Self {
        Self { x: tup.0, y: tup.1, z: tup.2, w: tup.3 }
    }

    /// convert a quaternion to an tuple.
    #[inline]
    pub const fn into_tuple(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.z, self.w)
    }

    /// create a quaternion with a given axis and angle value.
    #[inline]
    pub fn from_angle_axis(angle_radian: f32, axis: Vec3) -> Self {
        debug_assert!(axis.is_normalized(), "Axis must be normalized vector.");
        let (s, c) = (angle_radian * 0.5).sin_cos();
        Self {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: c
        }
    }

    /// create a quaternion with a given matrix.
    #[inline]
    pub fn from_matrix3x3(m: Mat3x3) -> Self {
        if m.r3c3 <= 0.0 {
            if m.r2c2 - m.r1c1 <= 0.0 {
                let t = 1.0 + m.r1c1 - m.r2c2 - m.r3c3;
                let inv = 0.5 / t.sqrt();
                Self {
                    x: t * inv,
                    y: (m.r1c2 + m.r2c1) * inv,
                    z: (m.r1c3 + m.r3c1) * inv,
                    w: (m.r2c3 - m.r3c2) * inv
                }
            }
            else {
                let t = 1.0 - m.r1c1 + m.r2c2 - m.r3c3;
                let inv = 0.5 / t.sqrt();
                Self {
                    x: (m.r1c2 + m.r2c1) * inv,
                    y: t * inv,
                    z: (m.r2c3 + m.r3c2) * inv,
                    w: (m.r3c1 - m.r1c3) * inv
                }
            }
        }
        else {
            if m.r1c1 + m.r2c2 <= 0.0 {
                let t = 1.0 - m.r1c1 - m.r2c2 + m.r3c3;
                let inv = 0.5 / t.sqrt();
                Self {
                    x: (m.r1c3 + m.r3c1) * inv,
                    y: (m.r2c3 + m.r3c2) * inv,
                    z: t * inv,
                    w: (m.r1c2 - m.r2c1) * inv
                }
            }
            else {
                let t = 1.0 + m.r1c1 + m.r2c2 + m.r3c3;
                let inv = 0.5 / t.sqrt();
                Self {
                    x: (m.r2c3 - m.r3c2) * inv,
                    y: (m.r3c1 - m.r1c3) * inv,
                    z: (m.r1c2 - m.r2c1) * inv,
                    w: t * inv
                }
            }
        }
    }

    /// convert a quaternion to an matrix.
    #[inline]
    pub fn into_matrix3x3(self) -> Mat3x3 {
        Mat3x3::from_quat(self)
    }

    /// create a quaternion with a given matrix.
    #[inline]
    pub fn from_matrix4x4(m: Mat4x4) -> Self {
        if m.r3c3 <= 0.0 {
            if m.r2c2 - m.r1c1 <= 0.0 {
                let t = 1.0 + m.r1c1 - m.r2c2 - m.r3c3;
                let inv = 0.5 / t.sqrt();
                Self {
                    x: t * inv,
                    y: (m.r1c2 + m.r2c1) * inv,
                    z: (m.r1c3 + m.r3c1) * inv,
                    w: (m.r2c3 - m.r3c2) * inv
                }
            }
            else {
                let t = 1.0 - m.r1c1 + m.r2c2 - m.r3c3;
                let inv = 0.5 / t.sqrt();
                Self {
                    x: (m.r1c2 + m.r2c1) * inv,
                    y: t * inv,
                    z: (m.r2c3 + m.r3c2) * inv,
                    w: (m.r3c1 - m.r1c3) * inv
                }
            }
        }
        else {
            if m.r1c1 + m.r2c2 <= 0.0 {
                let t = 1.0 - m.r1c1 - m.r2c2 + m.r3c3;
                let inv = 0.5 / t.sqrt();
                Self {
                    x: (m.r1c3 + m.r3c1) * inv,
                    y: (m.r2c3 + m.r3c2) * inv,
                    z: t * inv,
                    w: (m.r1c2 - m.r2c1) * inv
                }
            }
            else {
                let t = 1.0 + m.r1c1 + m.r2c2 + m.r3c3;
                let inv = 0.5 / t.sqrt();
                Self {
                    x: (m.r2c3 - m.r3c2) * inv,
                    y: (m.r3c1 - m.r1c3) * inv,
                    z: (m.r1c2 - m.r2c1) * inv,
                    w: t * inv
                }
            }
        }
    }

    /// convert a quaternion to an matrix.
    #[inline]
    pub fn into_matrix4x4(self) -> Mat4x4 {
        Mat4x4::from_quat(self)
    }

    #[inline]
    pub fn add_quat(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w 
        }
    }

    #[inline]
    pub fn add_assign_quat(&mut self, rhs: Self) {
        *self = self.add_quat(rhs)
    }

    #[inline]
    pub fn sub_quat(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w 
        }
    }

    #[inline]
    pub fn sub_assign_quat(&mut self, rhs: Self) {
        *self = self.sub_quat(rhs)
    }

    #[inline]
    pub fn mul_scalar(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs 
        }
    }

    #[inline]
    pub fn mul_assign_scalar(&mut self, rhs: f32) {
        *self = self.mul_scalar(rhs)
    }

    #[inline]
    pub fn mul_quat(self, rhs: Self) -> Self {
        Self {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z,
            z: self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z 
        }
    }

    #[inline]
    pub fn mul_assign_quat(&mut self, rhs: Self) {
        *self = self.mul_quat(rhs)
    }

    #[inline]
    pub fn div_scalar(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs 
        }
    }

    #[inline]
    pub fn div_assign_scalar(&mut self, rhs: f32) {
        *self = self.div_scalar(rhs)
    }

    /// return conjugate quaternion.
    #[inline]
    pub fn conjugate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w 
        }
    }

    /// return inverse quaternion.
    #[inline]
    pub fn inverse(&self) -> Self {
        self.conjugate().div_scalar(self.length())
    }

    /// return inverse quaternion.
    pub fn fast_inverse(&self) -> Self {
        self.conjugate().div_scalar(self.length_squared())
    }

    /// dot product.
    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }

    /// the length of the quaternion.
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// the square of the length of the quaternion.
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// return normalized quaternion.
    #[inline]
    pub fn normalize(&self) -> Self {
        self.div_scalar(self.length())
    }

    /// return `true` if it is a normalized quaternion.
    #[inline]
    pub fn is_normalized(&self) -> bool {
        (self.length_squared() - 1.0).abs() <= f32::EPSILON
    }

    /// return `None` if quaternion cannot be normalized.
    #[inline]
    pub fn try_normalized(&self) -> Option<Self> {
        let length = self.length();
        if length > f32::EPSILON {
            return Some(self.div_scalar(length));
        }
        return None;
    }

    /// return `true` if any element of the quaternion has the value of infinity.
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.x.is_infinite() | self.y.is_infinite() | self.z.is_infinite() | self.w.is_infinite()
    }

    /// return `true` if all elements of the quaternion are neither infinite nor NaN.
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() & self.y.is_finite() & self.z.is_finite() & self.w.is_finite()
    }

    /// return `true` if any element of the quaternion has the value of NaN.
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.x.is_nan() | self.y.is_nan() | self.z.is_nan() | self.w.is_nan()
    }

    /// return `true` if the two quaternions are equal.
    #[inline]
    pub fn equal(&self, other: &Self) -> bool {
        let mut flag = true;
        for &num in (*self - *other).as_ref().iter() {
            flag &= num.abs() <= f32::EPSILON
        }
        return flag
    }

    /// return the smaller of the elements of two quaternion.
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
            w: self.w.min(other.w) 
        }
    }

    /// return the greater of the elements of two quaternion.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
            w: self.w.max(other.w) 
        }
    }

    /// round up the decimal places of the elements of a quaternion.
    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
            z: self.z.ceil(),
            w: self.w.ceil() 
        }
    }

    /// round down the decimal places of the elements of a quaternion.
    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
            w: self.w.floor() 
        }
    }

    /// round the decimal places of the elements of a quaternion.
    #[inline]
    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
            z: self.z.round(),
            w: self.w.round() 
        }
    }
}

impl ops::Add<Self> for Quat {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.add_quat(rhs)
    }
}

impl ops::AddAssign<Self> for Quat {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign_quat(rhs)
    }
}

impl ops::Sub<Self> for Quat {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_quat(rhs)
    }
}

impl ops::SubAssign<Self> for Quat {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign_quat(rhs)
    }
}

impl ops::Neg for Quat {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: self.x.neg(),
            y: self.y.neg(),
            z: self.z.neg(),
            w: self.w.neg() 
        }
    }
}

impl ops::Mul<f32> for Quat {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        self.mul_scalar(rhs)
    }
}

impl ops::MulAssign<f32> for Quat {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.mul_assign_scalar(rhs)
    }
}

impl ops::Mul<Self> for Quat {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_quat(rhs)
    }
}

impl ops::MulAssign<Self> for Quat {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.mul_assign_quat(rhs)
    }
}

impl ops::Div<f32> for Quat {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        self.div_scalar(rhs)
    }
}

impl ops::DivAssign<f32> for Quat {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.div_assign_scalar(rhs)
    }
}

impl ops::Index<usize> for Quat {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("index out of range.")
        }
    }
}

impl ops::IndexMut<usize> for Quat {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("index out of range.")
        }
    }
}

impl cmp::PartialEq for Quat {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

impl From<Vec4> for Quat {
    #[inline]
    fn from(vec: Vec4) -> Self {
        Self::from_vector4(vec)
    }
}

impl Into<Vec4> for Quat {
    #[inline]
    fn into(self) -> Vec4 {
        self.into_vector4()
    }
}

impl From<[f32; 4]> for Quat {
    #[inline]
    fn from(arr: [f32; 4]) -> Self {
        Self::from_array(arr)
    }
}

impl Into<[f32; 4]> for Quat {
    #[inline]
    fn into(self) -> [f32; 4] {
        self.into_array()
    }
}

impl From<(f32, f32, f32, f32)> for Quat {
    #[inline]
    fn from(tup: (f32, f32, f32, f32)) -> Self {
        Self::from_tuple(tup)
    }
}

impl Into<(f32, f32, f32, f32)> for Quat {
    #[inline]
    fn into(self) -> (f32, f32, f32, f32) {
        self.into_tuple()
    }
}

impl From<Mat3x3> for Quat {
    #[inline]
    fn from(m: Mat3x3) -> Self {
        Self::from_matrix3x3(m)
    }
}

impl Into<Mat3x3> for Quat {
    #[inline]
    fn into(self) -> Mat3x3 {
        self.into_matrix3x3()
    }
}

impl From<Mat4x4> for Quat {
    #[inline]
    fn from(m: Mat4x4) -> Self {
        Self::from_matrix4x4(m)
    }
}

impl Into<Mat4x4> for Quat {
    #[inline]
    fn into(self) -> Mat4x4 {
        self.into_matrix4x4()
    }
}

impl AsRef<[f32; 4]> for Quat {
    #[inline]
    fn as_ref(&self) -> &[f32; 4] {
        unsafe { &*(self as *const Self as *const [f32; 4]) }
    }
}

impl AsMut<[f32; 4]> for Quat {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 4] {
        unsafe { &mut *(self as *mut Self as *mut [f32; 4]) }
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
