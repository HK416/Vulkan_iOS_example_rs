mod camera;
mod models;

use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
use bytemuck::{offset_of, Zeroable, Pod};
use crate::math::*;
use crate::timer::*;
use crate::renderer::*;
use crate::{err, error::RuntimeError};
use crate::world::mesh::{ModelMesh, self};
use crate::world::model::{Model, CameraModel, DynamicModel, DrawableModel};
use crate::world::scene::{SceneNode, SceneManager};
use crate::world::shader::ModelGraphicsShader;

use self::camera::PerspectiveCamera;
use self::models::{TriangleModel, TriangleMesh, Vertex};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneID {
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderID {
    Triangle,
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Zeroable, Pod)]
struct UniformBufferData {
    view_mtx: Mat4x4,
    projection_mtx: Mat4x4,
}

pub struct MainScene {
    _models: Vec<Arc<Mutex<dyn Model>>>,
    dynamic_models: VecDeque<Arc<Mutex<dyn DynamicModel>>>,
    drawable_models: HashMap<ShaderID, VecDeque<Arc<Mutex<dyn DrawableModel>>>>,
    shaders: HashMap<ShaderID, Arc<ModelGraphicsShader>>,
    camera: Arc<Mutex<dyn CameraModel>>,
    uniform_buffer_pool: CpuBufferPool<UniformBufferData>,
}

impl MainScene {
    pub fn new<C>(renderer: &Renderer, builder: &mut AutoCommandBufferBuilder<C>) -> Result<Box<Self>, RuntimeError> {
        let mut models = Vec::new();
        let mut dynamic_models = VecDeque::new();
        let mut drawable_models = HashMap::new();
        let mut shaders = HashMap::new();

        let assets_dir = renderer.ref_assets_dir();
        let file_path = PathBuf::from_iter([assets_dir.clone(), Path::new("shaders/vert.spv")]);
        let vertex_shader = renderer.load_shader_from_spv_file(&file_path)?;
        let file_path = PathBuf::from_iter([assets_dir.clone(), Path::new("shaders/frag.spv")]);
        let fragment_shader = renderer.load_shader_from_spv_file(&file_path)?;
        let triangle_shader = ModelGraphicsShader::new(
            renderer,
            Some(InputAssemblyState::new().topology(PrimitiveTopology::TriangleList)),
            Some(VertexInputState {
                attributes: [
                    (0, VertexInputAttributeDescription {
                        binding: 0,
                        format: Format::R32G32B32A32_SFLOAT,
                        offset: offset_of!(Vertex, color) as u32,
                    }),
                    (1, VertexInputAttributeDescription {
                        binding: 0,
                        format: Format::R32G32B32_SFLOAT,
                        offset: offset_of!(Vertex, position) as u32,
                    })
                ].into_iter().collect(),
                bindings: [
                    (0, VertexInputBindingDescription {
                        input_rate: VertexInputRate::Vertex,
                        stride: std::mem::size_of::<Vertex>() as u32
                    })
                ].into_iter().collect()
            }), 
            None,
            None,
            None,
            Some(
                RasterizationState::new()
                    .cull_mode(CullMode::None)
                    .front_face(FrontFace::CounterClockwise)
            ), 
            Some(
                ViewportState::viewport_dynamic_scissor_irrelevant()
            ),
            None,
            None,
            vertex_shader.entry_point("main").unwrap(),
            None,
            None,
            fragment_shader.entry_point("main").unwrap()
        )?;
        shaders.insert(ShaderID::Triangle, triangle_shader.clone());
        drawable_models.insert(ShaderID::Triangle, VecDeque::new());
        
        let triangle_mesh = TriangleMesh::new(renderer, builder)?;
        let triangle_model0 = TriangleModel::new(
            Vec3::new_vector(1.0, 0.0, 0.0), 
            Quat::IDENTITY, 
            triangle_mesh.clone()
        )?;
        models.push(triangle_model0.clone() as _);
        dynamic_models.push_back(triangle_model0.clone() as _);
        drawable_models.get_mut(&ShaderID::Triangle).unwrap().push_back(triangle_model0.clone() as _);

        let triangle_model1 = TriangleModel::new(
            Vec3::new_vector(-1.0, 0.0, 0.0),
            Quat::IDENTITY,
            triangle_mesh.clone()
        )?;
        models.push(triangle_model1.clone() as _);
        dynamic_models.push_back(triangle_model1.clone() as _);
        drawable_models.get_mut(&ShaderID::Triangle).unwrap().push_back(triangle_model1.clone() as _);

        let extent = renderer.get_swapchain_image_extent();
        let camera = PerspectiveCamera::new(
            Scissor { origin: [0, 0], dimensions: [extent[0], extent[1]] },
            Viewport { origin: [0.0, 0.0], dimensions: [extent[0] as f32, extent[1] as f32], depth_range: (0.0..1.0) },
            Vec3::new_vector(0.0, 0.0, 5.0),
            Quat::IDENTITY
        );

        let uniform_buffer_pool = renderer.create_cpu_buffer_pool(
            BufferUsage {
                uniform_buffer: true,
                ..Default::default()
            }, 
            MemoryUsage::Upload
        );

        Ok(Box::new(Self {
            _models: models,
            dynamic_models,
            drawable_models,
            shaders,
            camera,
            uniform_buffer_pool,
        }))
    }
}

impl SceneNode<SceneID> for MainScene {
    fn update(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> {
        for model in self.dynamic_models.iter() {
            model.lock().unwrap().update(timer)?;
        }
        Ok(())
    }

    fn draw(
        &mut self, 
        renderer: &Renderer,
        builder: &mut CmdBufBeginRenderPassGuard
    ) -> Result<(), RuntimeError> {
        let mut command_buffer_builder = renderer.secondary_command_buffer(
            Some(CommandBufferUsage::OneTimeSubmit),
            Some(builder.inheritance_info())
        )?;

        let (viewport, uniform_buffer) = {
            let camera = self.camera.lock().unwrap();
            let viewport = camera.ref_viewport().clone();
            let uniform_buffer = self.uniform_buffer_pool
                .from_data(UniformBufferData {
                    view_mtx: camera.get_view_matrix(),
                    projection_mtx: camera.get_projection_matrix()
                }
            ).map_err(|e| err!("Uniform Buffer Creation Failed: {}", e.to_string()))?;
            Ok((viewport, uniform_buffer))
        }?;

        command_buffer_builder.set_viewport(0, [viewport]);
        for (id, shader) in self.shaders.iter() {
            shader.bind_pipeline_graphics(&mut command_buffer_builder);
            let descriptor_set = shader.create_descriptor_set(
                renderer, 
                [WriteDescriptorSet::buffer(0, uniform_buffer.clone())]
            )?;
            shader.bind_descriptor_sets(0, descriptor_set, &mut command_buffer_builder);
            if let Some(models) = self.drawable_models.get(id) {
                shader.draw_models(models.iter(), &mut command_buffer_builder)?;

                // DEBUG-----------------------------
                for model in models.iter() {
                    let pos = model.lock().unwrap().get_position();
                    let pos = Vec4::new_vector(pos.x, pos.y, pos.z, 1.0);
                    let v = self.camera.lock().unwrap().get_view_matrix();
                    let p = self.camera.lock().unwrap().get_projection_matrix();
                    println!("{}", pos * v * p);
                }
            }
        }

        let command_buffer = command_buffer_builder
            .build()
            .map_err(|e| err!("Vk Command Buffer Builing Error: {}", e.to_string()))?;
        builder.execute_commands(command_buffer)
            .map_err(|e| err!("Vk Command Buffer Execution Failed Error: {}", e.to_string()))?;
        Ok(())
    }
}
