use std::ops::Neg;

pub use crate::math::*; 
pub use crate::renderer::*;
pub use crate::timer::Timer;
pub use crate::{err, error::RuntimeError};

use super::shader::ModelGraphicsShader;

pub trait Model {
    #[inline]
    fn get_position(&self) -> Vec3 {
        let mtx = self.ref_world_matrix();
        Vec3 { x: mtx.r4c1, y: mtx.r4c2, z: mtx.r4c3 }
    }

    #[inline]
    fn set_position(&mut self, position: Vec3) {
        let mtx = self.mut_world_matrix();
        mtx.r4c1 = position.x;
        mtx.r4c2 = position.y;
        mtx.r4c3 = position.z;
    }


    #[inline]
    fn get_quaternion(&self) -> Quat {
        self.ref_world_matrix().into_quat()
    }

    #[inline]
    fn set_quaternion(&mut self, quaternion: Quat) {
        assert!(quaternion.is_normalized(), "The given quaternion must be normalized.");
        let rm = quaternion.into_matrix3x3();
        let mtx = self.mut_world_matrix();
        
        mtx.r1c1 = rm.r1c1;
        mtx.r1c2 = rm.r1c2;
        mtx.r1c3 = rm.r1c3;

        mtx.r2c1 = rm.r2c1;
        mtx.r2c2 = rm.r2c2;
        mtx.r2c3 = rm.r2c3;

        mtx.r3c1 = rm.r3c1;
        mtx.r3c2 = rm.r3c2;
        mtx.r3c3 = rm.r3c3;
    }


    #[inline]
    fn get_right_vec(&self) -> Vec3 {
        let mtx = self.ref_world_matrix();
        Vec3 { x: mtx.r1c1, y: mtx.r1c2, z: mtx.r1c3 }
    }
    
    #[inline]
    fn get_up_vec(&self) -> Vec3 {
        let mtx = self.ref_world_matrix();
        Vec3 { x: mtx.r2c1, y: mtx.r2c2, z: mtx.r2c3 }
    }

    #[inline]
    fn get_look_vec(&self) -> Vec3 {
        let mtx = self.ref_world_matrix();
        Vec3 { x: mtx.r3c1, y: mtx.r3c2, z: mtx.r3c3 }
    }


    #[inline]
    fn translate_world(&mut self, distance: Vec3) {
        self.set_position(self.get_position() + distance);
    }

    #[inline]
    fn translate_local(&mut self, distance: Vec3) {
        let right = distance.x * self.get_right_vec();
        let up = distance.y * self.get_up_vec();
        let look = distance.z * self.get_look_vec();
        self.set_position(self.get_position() + right + up + look);
    }


    #[inline]
    fn rotate_quaternion(&mut self, quaternion: Quat) {
        assert!(quaternion.is_normalized(), "The given quaternion must be normalized.");
        self.set_quaternion((self.get_quaternion() * quaternion).normalize());
    }

    #[inline]
    fn rotate_angle_axis(&mut self, angle_radian: f32, axis: Vec3) {
        assert!(axis.is_normalized(), "The given axis must be normalized.");
        self.set_quaternion((self.get_quaternion() * Quat::from_angle_axis(angle_radian, axis)).normalize());
    }

    
    fn ref_world_matrix(&self) -> &Mat4x4;

    fn mut_world_matrix(&mut self) -> &mut Mat4x4;
}

pub trait DynamicModel : Model {
    fn update(&mut self, timer: &Timer) -> Result<(), RuntimeError>;
}

pub trait DrawableModel : Model {
    fn prepare_drawing(
        &mut self, 
        shader: &ModelGraphicsShader,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
    ) -> Result<(), RuntimeError>;

    fn draw(
        &mut self, 
        shader: &ModelGraphicsShader,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
    ) -> Result<(), RuntimeError>;
}

pub trait CameraModel : Model {
    fn ref_viewport(&self) -> &Viewport;

    fn mut_viewport(&mut self) -> &mut Viewport;

    fn ref_scissor(&self) -> &Scissor;

    fn mut_scissor(&mut self) -> &mut Scissor;

    #[inline]
    fn get_view_matrix(&self) -> Mat4x4 {
        let mut mtx = self.get_quaternion().into_matrix4x4().transpose();
        mtx.r1c3 *= -1.0;
        mtx.r2c3 *= -1.0;
        mtx.r3c3 *= -1.0;

        mtx.r4c1 = -self.get_position().dot(&self.get_right_vec());
        mtx.r4c2 = -self.get_position().dot(&self.get_up_vec());
        mtx.r4c3 = -self.get_position().dot(&self.get_look_vec());
        return mtx;
    }

    fn get_projection_matrix(&self) -> Mat4x4;
}
