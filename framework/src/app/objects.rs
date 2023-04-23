use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};

use crate::math::*;
use crate::timer::Timer;
use crate::world::model::*;
use crate::world::object::*;
use crate::world::variable::*;
use crate::renderer::RenderContext;
use crate::{err, error::RuntimeError};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct ObjectData {
    pub color: Vec4,
    pub transform: Mat4x4,
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct CameraData {
    pub view: Mat4x4,
    pub projection: Mat4x4,
}

pub struct Camera {
    pub mat: Mat4x4,
    pub screen_width: u32,
    pub screen_height: u32,
    pub uniform_buffer: Arc<UniformBuffer<CameraData>>,
}

impl GameObject for Camera { }

impl DrawAttributePrimary for Camera { }
impl DrawAttributeSecondary for Camera { }
impl DrawableObject for Camera { }

impl DynamicObject for Camera { 
    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    fn update(
        &mut self, 
        _elapsed_time_in_sec: f32, 
        _render_ctx: &Arc<RenderContext>
    ) -> Result<(), RuntimeError> {
        self.uniform_buffer.write_data(CameraData { 
            view: self.get_camera_mat(), 
            projection: self.get_projection_mat() 
        });

        Ok(())    
    }
}

impl WorldObject for Camera { 
    #[inline]
    fn ref_transform(&self) -> &Mat4x4 {
        &self.mat
    }

    #[inline]
    fn mut_transform(&mut self) -> &mut Mat4x4 {
        &mut self.mat
    }
}

impl CameraObject for Camera {
    fn get_camera_mat(&self) -> Mat4x4 {
        let mut mat = self.get_quaternion().into_matrix4x4().transpose();
        mat.r4c1 = - self.get_right_vector().dot(&self.get_position());
        mat.r4c2 = - self.get_up_vector().dot(&self.get_position());
        mat.r4c3 = - self.get_look_vector().dot(&self.get_position());
        return mat;
    }

    fn get_projection_mat(&self) -> Mat4x4 {
        perspective_lh_zo(
            60_f32.to_radians(), 
            self.screen_width as f32 / self.screen_height as f32,
            0.001, 
            1000.0
        )
    }
}



pub struct RotateObject {
    pub mat: Mat4x4,
    pub color: Vec4,
    pub axis: Vec3,
    pub speed: f32,
    pub model: Model,
}

impl GameObject for RotateObject { }

impl WorldObject for RotateObject {
    fn set_position(&mut self, position: Vec3) {
        let mut mat = self.mut_transform();
        mat.r4c1 = position.x;
        mat.r4c2 = position.y;
        mat.r4c3 = position.z;

        self.model.update_transform(&"Root".to_string(), Some(self.mat));
    }

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

        self.model.update_transform(&"Root".to_string(), Some(self.mat));
    }

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

        self.model.update_transform(&"Root".to_string(), Some(self.mat));
    }

    fn translate_world(&mut self, distance: Vec3) {
        let mut mat = self.mut_transform();
        mat.r4c1 += distance.x;
        mat.r4c2 += distance.y;
        mat.r4c3 += distance.z;
        
        self.model.update_transform(&"Root".to_string(), Some(self.mat));
    }

    fn rotate_from_quaternion(&mut self, quaternion: Quat) {
        let rot = quaternion.normalize().into_matrix4x4();
        let mat = self.mut_transform();
        *mat = rot * mat.clone();

        self.model.update_transform(&"Root".to_string(), Some(self.mat));
    }

    fn rotate_from_angle_axis(&mut self, angle: f32, axis: Vec3) {
        let rot = Quat::from_angle_axis(angle, axis.normalize()).into_matrix4x4();
        let mat = self.mut_transform();
        *mat = rot * mat.clone();
        
        self.model.update_transform(&"Root".to_string(), Some(self.mat));
    }
    
    #[inline]
    fn ref_transform(&self) -> &Mat4x4 {
        &self.mat
    }

    #[inline]
    fn mut_transform(&mut self) -> &mut Mat4x4 {
        &mut self.mat
    }
}

impl DrawAttributePrimary for RotateObject {
    fn draw(
        &self, 
        _render_ctx: &Arc<RenderContext>, 
        command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>
    ) -> Result<(), RuntimeError> {
        let nodes = self.model.ref_nodes();
        for node in nodes {
            if let Some(shader) = &node.shader {
                unsafe { 
                    shader.bind_pipeline(command_buffer_builder);
                    shader.bind_descriptor_set(command_buffer_builder);
                    shader.push_constants(
                        0, 
                        ObjectData {
                            color: self.color,
                            transform: node.world_matrix,
                        }, 
                        command_buffer_builder
                    );
                }
            }

            if let Some(mesh) = &node.mesh {
                unsafe {
                    mesh.bind_buffers(command_buffer_builder);
                    mesh.draw(1, 0, command_buffer_builder)?;
                }
            }
        }

        Ok(())    
    }
}

impl DrawAttributeSecondary for RotateObject {
    fn darw(
        &self, 
        _render_ctx: &Arc<RenderContext>, 
        command_buffer_builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
    ) -> Result<(), RuntimeError> {
        let nodes = self.model.ref_nodes();
        for node in nodes {
            if let Some(shader) = &node.shader {
                unsafe { 
                    shader.bind_pipeline(command_buffer_builder);
                    shader.bind_descriptor_set(command_buffer_builder);
                    shader.push_constants(
                        0, 
                        ObjectData {
                            color: self.color,
                            transform: node.world_matrix,
                        }, 
                        command_buffer_builder
                    );
                }
            }

            if let Some(mesh) = &node.mesh {
                unsafe {
                    mesh.bind_buffers(command_buffer_builder);
                    mesh.draw(1, 0, command_buffer_builder)?;
                }
            }
        }

        Ok(())
    }
}

impl DrawableObject for RotateObject {
    #[inline]
    fn is_visible(&self) -> bool {
        true
    }
}

impl DynamicObject for RotateObject {
    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    fn update(
        &mut self, 
        elapsed_time_in_sec: f32, 
        _render_ctx: &Arc<RenderContext>
    ) -> Result<(), RuntimeError> {
        self.rotate_from_angle_axis(
            45_f32.to_radians() * self.speed * elapsed_time_in_sec, 
            self.axis
        );
        Ok(())
    }
}
