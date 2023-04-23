mod id;
mod objects;
mod constant;

use std::any::Any;
use std::collections::VecDeque;
use std::fmt;
use std::mem;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Condvar};
use std::collections::HashMap;

use rand::prelude::*;
use bytemuck::{Pod, Zeroable};
use vulkano::buffer::allocator;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferInheritanceInfo;
use vulkano::command_buffer::CommandBufferInheritanceRenderPassInfo;
use vulkano::command_buffer::CommandBufferInheritanceRenderPassType;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::PrimaryCommandBufferAbstract;
use vulkano::command_buffer::RenderPassBeginInfo;
use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use vulkano::command_buffer::SubpassContents;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocatorCreateInfo;
use vulkano::format::ClearValue;
use vulkano::format::Format;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::StateMode;
use vulkano::pipeline::graphics::color_blend::AttachmentBlend;
use vulkano::pipeline::graphics::color_blend::BlendFactor;
use vulkano::pipeline::graphics::color_blend::BlendOp;
use vulkano::pipeline::graphics::color_blend::ColorBlendAttachmentState;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::color_blend::ColorComponents;
use vulkano::pipeline::graphics::color_blend::LogicOp;
use vulkano::pipeline::graphics::depth_stencil::CompareOp;
use vulkano::pipeline::graphics::depth_stencil::DepthState;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::rasterization::FrontFace;
use vulkano::pipeline::graphics::rasterization::PolygonMode;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::VertexInputAttributeDescription;
use vulkano::pipeline::graphics::vertex_input::VertexInputBindingDescription;
use vulkano::pipeline::graphics::vertex_input::VertexInputRate;
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::render_pass::Subpass;
use vulkano::sync::GpuFuture;

use crate::math::*;
use crate::timer::*;
use crate::renderer::*;
use crate::world::mesh;
use crate::world::mesh::*;
use crate::world::model::*;
use crate::world::scene::*;
use crate::world::shader;
use crate::world::shader::*;
use crate::world::object::*;
use crate::world::variable::*;
use crate::{err, error::RuntimeError};

use self::id::*;
use self::objects::*;
use self::constant::*;


pub struct MainScene {
    camera: Option<Camera>,
    objects: Vec<Arc<Mutex<dyn WorldObject>>>,
}

impl MainScene {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            camera: None,
            objects: Vec::with_capacity(MAX_OBJECTS_NUM),
        })
    }
}

impl SceneNode<String> for MainScene {
    fn enter(&mut self, renderer: &Renderer) -> Result<(), RuntimeError> {
        // create triangle mesh.
        let render_ctx = renderer.ref_render_context().clone();
        let triangle_mesh = thread::spawn(move || {
            create_triangle_mesh(render_ctx)
        });

        // create quad mesh.
        let render_ctx = renderer.ref_render_context().clone();
        let quad_mesh = thread::spawn(move || {
            create_quad_mesh(render_ctx)
        });

        // create cube mesh.
        let render_ctx = renderer.ref_render_context().clone();
        let cube_mesh = thread::spawn(move || {
            create_cube_mesh(render_ctx)
        });

        // load shader module
        let assets_dir = renderer.ref_assets_dir().to_path_buf();
        let render_ctx = renderer.ref_render_context().clone();
        let vs = thread::spawn(move || {
            let path = PathBuf::from_iter([ assets_dir, PathBuf::from(VERT_SHADER_PATH) ]);
            load_from_spv_file(&path, &render_ctx)
        });
        let assets_dir = renderer.ref_assets_dir().to_path_buf();
        let render_ctx = renderer.ref_render_context().clone();
        let fs = thread::spawn(move || {
            let path = PathBuf::from_iter([ assets_dir, PathBuf::from(FRAG_SHADER_PATH) ]);
            load_from_spv_file(&path, &render_ctx)
        });

        // create a graphics pipeline.
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(
                VertexInputState::new()
                    .binding(0, VertexInputBindingDescription {
                        stride: mem::size_of::<Vec3>() as u32,
                        input_rate: VertexInputRate::Vertex,
                    })
                    .attribute(0, VertexInputAttributeDescription {
                        binding: 0,
                        offset: 0,
                        format: Format::R32G32B32_SFLOAT,
                    })
            )
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .vertex_shader(vs.join().unwrap()?.entry_point("main").unwrap(), ())
            .fragment_shader(fs.join().unwrap()?.entry_point("main").unwrap(), ())
            .render_pass(renderer.pipeline_begin_render_pass_type(0).unwrap())
            .build_with_cache(renderer.ref_pipeline_cache().clone())
            .build(renderer.ref_render_context().ref_device().clone())
            .map_err(|e| err!("Graphics pipeline creation failed: {}", e.to_string()))?;


        // create the shader variable.
        let render_ctx = renderer.ref_render_context().clone();
        let uniform_buffer: Arc<UniformBuffer<CameraData>> = UniformBuffer::from_data(
            CameraData { view: Mat4x4::IDENTITY, projection: Mat4x4::IDENTITY },
            render_ctx.ref_memory_allocator(),
        )?;

        
        // create a camera object.
        let mut camera = Camera {
            mat: Mat4x4::IDENTITY,
            screen_width: renderer.get_screen_size().0,
            screen_height: renderer.get_screen_size().1,
            uniform_buffer: uniform_buffer.clone(),
        };

        camera.set_position(Vec3::new_vector(0.0, 0.0, -10.0));
        camera.set_look_at_point(Vec3::ZERO);

        self.camera = Some(camera);


        // create a graphics shader.
        let default_shader = GraphicsShader::new(
            pipeline, 
            render_ctx.ref_descriptor_allocator(), 
            [uniform_buffer.clone() as _]
        )?;

        // create game objects.
        let shaders = HashMap::from([(ShaderID::Default, default_shader)]);
        let mut meshes = HashMap::new();
        let mut command_buffers = Vec::new();

        let (mesh, command_buffer) = triangle_mesh.join().unwrap()?;
        meshes.insert(MeshID::Triangle, mesh);
        command_buffers.push(command_buffer);

        let (mesh, command_buffer) = quad_mesh.join().unwrap()?;
        meshes.insert(MeshID::Quad, mesh);
        command_buffers.push(command_buffer);

        let (mesh, command_buffer) = cube_mesh.join().unwrap()?;
        meshes.insert(MeshID::Cube, mesh);
        command_buffers.push(command_buffer);

        let objects = thread::spawn(move || {
            create_game_objects(meshes, shaders)
        });


        let render_ctx = renderer.ref_render_context().clone();
        let allocator = render_ctx.get_command_buffer_allocator();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &allocator, 
            render_ctx.get_queue_fmaily_index(), 
            CommandBufferUsage::OneTimeSubmit
        ).map_err(|e| err!("Primary command buffer begining failed: {}", e.to_string()))?;

        command_buffer_builder
            .execute_commands_from_vec(command_buffers)
            .map_err(|e| err!("Secondary command buffer execution failed: {}", e.to_string()))?;
        let command_buffer = command_buffer_builder.build()
            .map_err(|e| err!("Primary command buffer building failed: {}", e.to_string()))?;

        command_buffer
            .execute(render_ctx.ref_integrated_queue().clone())
            .map_err(|e| err!("Primary command buffer execution failed: {}", e.to_string()))?
            .then_signal_fence_and_flush()
            .map_err(|e| err!("Primary command buffer flush failed: {}", e.to_string()))?
            .wait(None)
            .map_err(|e| err!("Primary command buffer flush failed: {}", e.to_string()))?;

        self.objects = objects.join().unwrap();
        Ok(())
    }

    fn update(&mut self, timer: &Timer, renderer: &Renderer) -> Result<(), RuntimeError> {
        let elapsed_time_in_sec = timer.get_elapsed_time_in_sec();

        if let Some(camera) = &mut self.camera {
            if camera.is_dynamic() {
                camera.update(elapsed_time_in_sec, renderer.ref_render_context())?;
            }
        }

        let num_threads = renderer.get_num_threads();
        let object_range = MAX_OBJECTS_NUM / num_threads;
        let mut handles = Vec::with_capacity(num_threads);
        for i in 0..renderer.get_num_threads() {
            let objects = self.objects.clone();
            let render_ctx = renderer.ref_render_context().clone();
            handles.push(thread::spawn(move || -> Result<(), RuntimeError> {
                for idx in object_range * i..object_range * (i + 1) {
                    objects[idx].lock().unwrap().update(elapsed_time_in_sec, &render_ctx)?;
                }

                Ok(())
            }));
        }

        while let Some(handle) = handles.pop() {
            handle.join().unwrap()?;
        }

        Ok(())
    }

    fn draw(&mut self, renderer: &mut Renderer) -> Result<(), RuntimeError> {
        // wait for next frame.
        let (acquire_future, framebuffer) = match renderer.wait_for_next_frame()? {
            Some(it) => it,
            None => return Ok(())
        };

        // create a primary command buffer.
        let render_ctx = renderer.ref_render_context().clone();
        let allocator = render_ctx.get_command_buffer_allocator();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &allocator, 
            render_ctx.get_queue_fmaily_index(), 
            CommandBufferUsage::OneTimeSubmit
        ).map_err(|e| err!("Command buffer begining failed: {}", e.to_string()))?;

        // begin render pass.
        command_buffer_builder.begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some(ClearValue::Float([1.0, 1.0, 1.0, 1.0])),
                    Some(ClearValue::DepthStencil((1.0, 0)))
                ],
                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            }, 
            SubpassContents::SecondaryCommandBuffers
        ).map_err(|e| err!("Render pass begining failed: {}", e.to_string()))?;
        let inheritance_info = CommandBufferInheritanceInfo {
            render_pass: Some(
                CommandBufferInheritanceRenderPassType::BeginRenderPass(
                    CommandBufferInheritanceRenderPassInfo {
                        framebuffer: Some(framebuffer.clone()),
                        subpass: Subpass::from(framebuffer.render_pass().clone(), 0).unwrap()
                    }
                )
            ),
            ..Default::default()
        };

        // muti-thread rendering
        let num_threads = renderer.get_num_threads();
        let object_range = MAX_OBJECTS_NUM / num_threads;
        let mut handles = Vec::with_capacity(num_threads);
        for i in 0..renderer.get_num_threads() {
            let screen_size = renderer.get_screen_size();
            let render_ctx = renderer.ref_render_context().clone();
            // let jobs_cp = jobs.clone();
            let objects = self.objects.clone();
            let inheritance_info_cp = inheritance_info.clone();
            handles.push(thread::spawn(move || -> Result<SecondaryAutoCommandBuffer, RuntimeError> {
                let allocator = render_ctx.get_command_buffer_allocator();
                let mut command_buffer_builder = AutoCommandBufferBuilder::secondary(
                    &allocator, 
                    render_ctx.get_queue_fmaily_index(), 
                    CommandBufferUsage::OneTimeSubmit, 
                    inheritance_info_cp,
                ).map_err(|e| err!("Secondary command buffer begining failed: {}", e.to_string()))?;

                // set viewport
                command_buffer_builder.set_viewport(0, [Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [screen_size.0 as f32, screen_size.1 as f32],
                    depth_range: (0.0..1.0)
                }]);

                for idx in object_range * i..object_range * (i + 1) {
                    objects[idx].lock().unwrap().darw(&render_ctx, &mut command_buffer_builder)?;
                }

                Ok(command_buffer_builder
                    .build()
                    .map_err(|e| err!("Secondary command buffer building failed: {}", e.to_string()))?)
            }));
        }

        let mut command_buffers = Vec::with_capacity(handles.capacity());
        while let Some(handle) = handles.pop() {
            command_buffers.push(handle.join().unwrap()?);
        }

        // command buffer building.
        command_buffer_builder.execute_commands_from_vec(command_buffers)
            .map_err(|e| err!("Primary command buffer execution failed: {}", e.to_string()))?
            .end_render_pass()
            .map_err(|e| err!("Primary command buffer recoring failed: {}", e.to_string()))?;
        
        let command_buffer = command_buffer_builder.build()
            .map_err(|e| err!("Primary command buffer building failed: {}", e.to_string()))?;

        // queue submit and present.
        renderer.queue_submit_and_present(acquire_future, command_buffer)?;
        Ok(())
    }
}

impl fmt::Debug for MainScene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MainScene").finish()
    }
}


#[inline]
fn create_game_objects(
    meshes: HashMap<MeshID, Arc<Mesh>>, 
    shaders: HashMap<ShaderID, Arc<GraphicsShader>>
) -> Vec<Arc<Mutex<dyn WorldObject>>> {
    let mut rng = thread_rng();
    let mut objects = Vec::with_capacity(MAX_OBJECTS_NUM);
    for _ in 0..MAX_OBJECTS_NUM {
        let position = Vec3::new_vector(
            rng.gen_range(-100.0..=100.0),
            rng.gen_range(-100.0..=100.0),
            rng.gen_range(-100.0..=100.0)
        );

        let axis = Vec3::new_vector(
            rng.gen_range(-1.0..=1.0), 
            rng.gen_range(-1.0..=1.0), 
            rng.gen_range(-1.0..=1.0)
        ).normalize();

        let speed: f32 = rng.gen_range(-1.0..=1.0);

        let color = Vec4::new_vector(
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
        );

        let q = Quat::from_angle_axis(0.0, axis);
        let mut mat = q.normalize().into_matrix4x4();
        mat.r4c1 = position.x;
        mat.r4c2 = position.y;
        mat.r4c3 = position.z;

        let mesh = meshes.get(&rand::random()).unwrap().clone();
        let shader = shaders.get(&rand::random()).unwrap().clone();
        let model_node = ModelNode {
            id: "Root".to_string(),
            transform: Mat4x4::IDENTITY,
            world_matrix: mat,
            mesh: Some(mesh),
            shader: Some(shader),
            parent: None,
            sibling: None,
            child: None
        };
        let model = Model::from_nodes(
            "Unknown",
            "Root".to_string(),
            [model_node]
        ).unwrap();

        objects.push(match rand::random() {
            SystemID::Rotation => { 
                Arc::new(Mutex::new(RotateObject {
                    mat,
                    color,
                    axis,
                    speed,
                    model
                })) as _
            }
        });
    }
    return objects;
}


#[inline]
fn create_triangle_mesh(
    render_ctx: Arc<RenderContext>
) -> Result<(Arc<Mesh>, SecondaryAutoCommandBuffer), RuntimeError> {
    // create secondary command buffer.
    let allocator = render_ctx.get_command_buffer_allocator();
    let mut command_buffer_builder = AutoCommandBufferBuilder::secondary(
        &allocator, 
        render_ctx.get_queue_fmaily_index(), 
        CommandBufferUsage::OneTimeSubmit,
        CommandBufferInheritanceInfo::default()
    ).map_err(|e| err!("Secondary command buffer begining failed: {}", e.to_string()))?;

    // create vertex buffer.
    let positions = GpuVertexBuffer::from_iter_vec3(
        TRIANGLE_POSITIONS, 
        VertexInputRate::Vertex, 
        render_ctx.ref_memory_allocator(), 
        &mut command_buffer_builder
    )? as _;

    // build command buffer.
    let command_buffer = command_buffer_builder
        .build()
        .map_err(|e| err!("Secondary command buffer building failed: {}", e.to_string()))?;

    Ok((
        Mesh::new(3, [positions]), 
        command_buffer
    ))
}


#[inline]
fn create_quad_mesh(
    render_ctx: Arc<RenderContext>
) -> Result<(Arc<Mesh>, SecondaryAutoCommandBuffer), RuntimeError> {
    // create secondary command buffer.
    let allocator = render_ctx.get_command_buffer_allocator();
    let mut command_buffer_builder = AutoCommandBufferBuilder::secondary(
        &allocator, 
        render_ctx.get_queue_fmaily_index(), 
        CommandBufferUsage::OneTimeSubmit, 
        CommandBufferInheritanceInfo::default()
    ).map_err(|e| err!("Secondary command buffer begining failed: {}", e.to_string()))?;

    // create index buffer.
    let index_buffer = IndexBuffer::from_iter_u16(
        QUAD_INDICES,
        render_ctx.ref_memory_allocator(),
        &mut command_buffer_builder
    )?;

    // create vertex buffer.
    let positions = GpuVertexBuffer::from_iter_vec3(
        QUAD_POSITIONS,
        VertexInputRate::Vertex,
        render_ctx.ref_memory_allocator(),
        &mut command_buffer_builder
    )? as _;

    // build command buffer.
    let command_buffer = command_buffer_builder
        .build()
        .map_err(|e| err!("Secondary command buffer building failed: {}", e.to_string()))?;

    Ok((
        Mesh::new_with_index(6, index_buffer, 4, [positions]), 
        command_buffer
    ))
}


#[inline]
fn create_cube_mesh(
    render_ctx: Arc<RenderContext>
) -> Result<(Arc<Mesh>, SecondaryAutoCommandBuffer), RuntimeError> {
    // create secondary command buffer.
    let allocator = render_ctx.get_command_buffer_allocator();
    let mut command_buffer_builder = AutoCommandBufferBuilder::secondary(
        &allocator, 
        render_ctx.get_queue_fmaily_index(), 
        CommandBufferUsage::OneTimeSubmit, 
        CommandBufferInheritanceInfo::default()
    ).map_err(|e| err!("Secondary command buffer begining failed: {}", e.to_string()))?;

    // create index buffer.
    let index_buffer = IndexBuffer::from_iter_u16(
        CUBE_INDICES,
        render_ctx.ref_memory_allocator(),
        &mut command_buffer_builder
    )?;

    // create vertex buffer.
    let positions = GpuVertexBuffer::from_iter_vec3(
        CUBE_POSITIONS,
        VertexInputRate::Vertex,
        render_ctx.ref_memory_allocator(),
        &mut command_buffer_builder
    )? as _;

    // build command buffer.
    let command_buffer = command_buffer_builder
        .build()
        .map_err(|e| err!("Secondary command buffer building failed: {}", e.to_string()))?;

    
    Ok((
        Mesh::new_with_index(36, index_buffer, 8,[positions]), 
        command_buffer
    ))
}
