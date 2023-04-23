use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};

use crate::math::*;
use crate::renderer::RenderContext;
use crate::{err, error::RuntimeError};


pub trait GameObject : Sync + Send { }

pub trait DrawAttributePrimary {
    fn draw(&self, _render_ctx: &Arc<RenderContext>, _command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<(), RuntimeError> { Ok(()) }
}

pub trait DrawAttributeSecondary {
    fn darw(&self, _render_ctx: &Arc<RenderContext>, _command_buffer_builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>) -> Result<(), RuntimeError> { Ok(()) }
}

pub trait DrawableObject : DrawAttributePrimary + DrawAttributeSecondary + GameObject {
    fn is_visible(&self) -> bool { false }
}


pub trait DynamicObject : GameObject {
    fn is_dynamic(&self) -> bool { false }
    fn update(&mut self, _elapsed_time_in_sec: f32, _render_ctx: &Arc<RenderContext>) -> Result<(), RuntimeError> { Ok(()) }
}


pub trait WorldObject : DrawableObject + DynamicObject {
    #[inline]
    fn get_position(&self) -> Vec3 {
        let mat = self.ref_transform();
        Vec3::new_vector(
            mat.r4c1, 
            mat.r4c2, 
            mat.r4c3
        )
    }

    #[inline]
    fn set_position(&mut self, position: Vec3) {
        let mut mat = self.mut_transform();
        mat.r4c1 = position.x;
        mat.r4c2 = position.y;
        mat.r4c3 = position.z;
    }

    #[inline]
    fn get_right_vector(&self) -> Vec3 {
        let mat = self.ref_transform();
        Vec3::new_vector(
            mat.r1c1, 
            mat.r1c2, 
            mat.r1c3
        ).normalize()
    }

    #[inline]
    fn get_up_vector(&self) -> Vec3 {
        let mat = self.ref_transform();
        Vec3::new_vector(
            mat.r2c1, 
            mat.r2c2, 
            mat.r2c3
        ).normalize()
    }

    #[inline]
    fn get_look_vector(&self) -> Vec3 {
        let mat = self.ref_transform();
        Vec3::new_vector(
            mat.r3c1, 
            mat.r3c2, 
            mat.r3c3
        ).normalize()
    }

    #[inline]
    fn get_quaternion(&self) -> Quat {
        self.ref_transform().into_quat().normalize()
    }

    #[inline]
    fn set_look_at_point(&mut self, point: Vec3) {
        assert!(point != self.get_position(), "A given point must not equal a position.");

        let up = self.get_up_vector().normalize();
        let look = (point - self.get_position()).normalize();
        let right = up.cross(&look).normalize();
        let up = look.cross(&right).normalize();

        let mut mat = self.mut_transform();
        mat.r1c1 = right.x;
        mat.r1c2 = right.y;
        mat.r1c3 = right.z;

        mat.r2c1 = up.x;
        mat.r2c2 = up.y;
        mat.r2c3 = up.z;

        mat.r3c1 = look.x;
        mat.r3c2 = look.y;
        mat.r3c3 = look.z;
    }

    #[inline]
    fn set_quaternion(&mut self, quaternion: Quat) {
        let rot = quaternion.normalize().into_matrix3x3();
        let mut mat = self.mut_transform();
        mat.r1c1 = rot.r1c1;
        mat.r1c2 = rot.r1c2;
        mat.r1c3 = rot.r1c3;

        mat.r2c1 = rot.r2c1;
        mat.r2c2 = rot.r2c2;
        mat.r2c3 = rot.r2c3;

        mat.r3c1 = rot.r3c1;
        mat.r3c2 = rot.r3c2;
        mat.r3c3 = rot.r3c3;
    }

    #[inline]
    fn translate_local(&mut self, distance: Vec3) {
        let x = self.get_right_vector() * distance.x;
        let y = self.get_up_vector() * distance.y;
        let z = self.get_look_vector() * distance.z;
        self.translate_world(x + y + z);
    }

    #[inline]
    fn translate_world(&mut self, distance: Vec3) {
        let mut mat = self.mut_transform();
        mat.r4c1 += distance.x;
        mat.r4c2 += distance.y;
        mat.r4c3 += distance.z;
    }

    #[inline]
    fn rotate_from_quaternion(&mut self, quaternion: Quat) {
        let rot = quaternion.normalize().into_matrix4x4();
        let mat = self.mut_transform();
        *mat = rot * mat.clone();
    }

    #[inline]
    fn rotate_from_angle_axis(&mut self, angle: f32, axis: Vec3) {
        let rot = Quat::from_angle_axis(angle, axis.normalize()).into_matrix4x4();
        let mat = self.mut_transform();
        *mat = rot * mat.clone();
    }

    fn ref_transform(&self) -> &Mat4x4;
    
    fn mut_transform(&mut self) -> &mut Mat4x4;
}


pub trait CameraObject : WorldObject {
    fn get_camera_mat(&self) -> Mat4x4;
    fn get_projection_mat(&self) -> Mat4x4;
}