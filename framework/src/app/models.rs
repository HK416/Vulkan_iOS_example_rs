use std::any::Any;
use std::sync::Arc;

use bytemuck::{Zeroable, Pod};

use crate::timer::Timer;
use crate::world::mesh::*;
use crate::world::model::*;
use crate::world::shader::*;
use crate::renderer::*;

#[derive(Clone)]
pub(crate) struct TriangleModel {
    transform: Mat4x4,
    mesh: Arc<dyn ModelMesh>
}

impl TriangleModel {
    #[inline]
    pub fn new(
        position: Vec3, 
        quaternion: Quat,
        mesh: Arc<dyn ModelMesh>,
    ) -> Result<Arc<Mutex<Self>>, RuntimeError> {
        let mut transform = quaternion.into_matrix4x4();
        transform.r4c1 = position.x;
        transform.r4c2 = position.y;
        transform.r4c3 = position.z;

        Ok(Arc::new(Mutex::new(Self { transform, mesh })))
    }
}

impl Model for TriangleModel {
    #[inline]
    fn ref_world_matrix(&self) -> &Mat4x4 {
        &self.transform
    }

    #[inline]
    fn mut_world_matrix(&mut self) -> &mut Mat4x4 {
        &mut self.transform
    }
}

impl DynamicModel for TriangleModel {
    fn update(&mut self, timer: &Timer) -> Result<(), RuntimeError> {
        let elapsed_time_in_sec = timer.get_elapsed_time_in_sec();
        self.rotate_angle_axis(60_f32.to_radians() * elapsed_time_in_sec, Vec3::Y);
        Ok(())
    }
}

impl DrawableModel for TriangleModel {
    fn prepare_drawing(
        &mut self, 
        shader: &ModelGraphicsShader,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
    ) -> Result<(), RuntimeError> {
        shader.push_constants(0, self.transform, builder);
        self.mesh.prepare_drawing(builder)?;
        Ok(())
    }

    fn draw(
        &mut self, 
        shader: &ModelGraphicsShader,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
    ) -> Result<(), RuntimeError> {
        self.mesh.draw(builder)?;
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Zeroable, Pod)]
pub(crate) struct Vertex {
    pub color: Vec4,
    pub position: Vec3,
}

#[derive(Debug, Clone)]
pub(crate) struct TriangleMesh {
    vertex_count: u32,
    vertex_buffer: Arc<DeviceLocalBuffer<[Vertex]>>,
}

impl TriangleMesh {
    pub fn new<L, A>(
        renderer: &Renderer, 
        builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<Self>, RuntimeError> 
    where A: CommandBufferAllocator {
        let vertices = [
            Vertex { 
                color: Vec4 { x: 1.0, y: 0.0, z: 0.0, w: 1.0 },
                position: Vec3 { x: -0.5, y: -0.25, z: 0.0 }
            },
            Vertex {
                color: Vec4 { x: 0.0, y: 1.0, z: 0.0, w: 1.0 },
                position: Vec3 { x: 0.5, y: -0.25, z: 0.0 },
            },
            Vertex {
                color: Vec4 { x: 0.0, y: 0.0, z: 1.0, w: 1.0 },
                position: Vec3 { x: 0.0, y: 0.5, z: 0.0 }
            },
        ];

        let vertex_buffer = renderer.create_device_local_buffer_from_iter(
            vertices, 
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            }, 
            builder
        )?;

        Ok(Arc::new(Self { vertex_count: 3, vertex_buffer }))
    }
}

impl ModelMesh for TriangleMesh {
    fn prepare_drawing(&self, builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>) -> Result<(), RuntimeError> {
        builder.bind_vertex_buffers(0, self.vertex_buffer.clone());
        Ok(())
    }

    fn draw(&self, builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>) -> Result<(), RuntimeError> {
        builder.draw(self.vertex_count, 1, 0, 0)
            .map_err(|e| err!("Vk Draw Error: {}", e.to_string()))?;
        Ok(())
    }
}
