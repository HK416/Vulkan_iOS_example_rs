mod platform;

mod frame;
mod context;
mod swapchain;
mod depth_stencil;

use std::{fs, thread};
use std::io::Read;
use std::sync::{Arc, Mutex, MutexGuard, Once};
use std::path::{Path, PathBuf};

use vulkano::command_buffer::{PrimaryAutoCommandBuffer, AutoCommandBufferBuilder, RenderPassBeginInfo};
use vulkano::command_buffer::allocator::{CommandBufferAlloc, CommandBufferAllocator};
use vulkano::format::Format;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::cache::PipelineCache;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::render_pass::{PipelineRenderingCreateInfo, PipelineRenderPassType};
use vulkano::pipeline::graphics::viewport::{ViewportState, Viewport};
use vulkano::pipeline::graphics::{GraphicsPipelineBuilder, rasterization};
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::render_pass::{Subpass, Framebuffer};
use vulkano::shader::{ShaderModule, EntryPoint, SpecializationConstants};
use vulkano::swapchain::SwapchainAcquireFuture;

use self::frame::RenderFrame;
use crate::{err, error::RuntimeError};

pub use self::platform::AppHandle;
pub use self::context::RenderContext;



#[derive(Debug)]
pub struct Renderer {
    num_threads: usize,

    handle: AppHandle,
    assets_dir: PathBuf,
    scale_factor: f32,
    screen_size: (u32, u32),
    viewer_area: (i32, i32, i32, i32),
    
    render_ctx: Arc<RenderContext>,
    render_frame: Arc<Mutex<RenderFrame>>,
    pipeline_cache: Arc<PipelineCache>,
}

impl Renderer {
    pub fn new(
        handle: AppHandle, 
        assets_dir: &Path,
        scale_factor: f32,
        screen_size: (u32, u32),
        viewer_area: (i32, i32, i32, i32),
    ) -> Result<Self, RuntimeError> {
        // create a new `RenderContext`
        let render_ctx = RenderContext::new(&handle)?;

        // create a new `RenderFrame`
        let render_frame = RenderFrame::new(
            (screen_size.0 as f32 * scale_factor) as u32, 
            (screen_size.1 as f32 * scale_factor) as u32, 
            &render_ctx
        )?;

        // create a new `PipelineCache`
        let pipeline_cache = PipelineCache::empty(
            render_ctx.ref_device().clone()
        ).map_err(|e| err!("Pipeline creation failed: {}", e.to_string()))?;

        // get number of threads.
        let num_threads = match thread::available_parallelism() {
            Ok(num) => usize::from(num),
            _ => 1,
        };

        Ok(Self { 
            num_threads,
            handle,
            assets_dir: assets_dir.to_path_buf(),
            scale_factor,
            screen_size,
            viewer_area,
            render_ctx,
            render_frame,
            pipeline_cache,
        })
    }


    #[inline]
    pub fn get_num_threads(&self) -> usize {
        self.num_threads
    }

    #[inline]
    pub fn get_screen_size(&self) -> (u32, u32) {
        (
            (self.screen_size.0 as f32 * self.scale_factor) as u32,
            (self.screen_size.1 as f32 * self.scale_factor) as u32,
        )
    }

    #[inline]
    pub fn get_viewer_area(&self) -> (i32, i32, i32, i32) {
        self.viewer_area
    }

    #[inline]
    pub fn ref_assets_dir(&self) -> &Path {
        &self.assets_dir
    }


    #[inline]
    pub fn ref_render_context(&self) -> &Arc<RenderContext> {
        &self.render_ctx
    }


    #[inline]
    pub fn wait_for_next_frame(&mut self) -> Result<Option<(SwapchainAcquireFuture, Arc<Framebuffer>)>, RuntimeError> {
        self.render_frame.lock().unwrap().wait_for_next_frame(
            self.scale_factor, 
            self.screen_size.0, 
            self.screen_size.1
        )
    }


    #[inline]
    pub fn queue_submit_and_present<A: CommandBufferAlloc>(
        &mut self,
        acquire_future: SwapchainAcquireFuture,
        command_buffer: PrimaryAutoCommandBuffer<A>
    ) -> Result<(), RuntimeError> {
        self.render_frame.lock().unwrap().queue_submit_and_present(
            &self.render_ctx, 
            acquire_future,
            command_buffer
        )
    }

    #[inline]
    pub fn ref_pipeline_cache(&self) -> &Arc<PipelineCache> {
        &self.pipeline_cache
    }

    #[inline]
    pub fn pipeline_begin_render_pass_type(
        &self,
        id: u32,
    ) -> Option<PipelineRenderPassType> {
        // clone render pass.
        let render_pass = {
            let guard = self.render_frame.lock().unwrap();
            guard.ref_render_pass().clone()
        };

        if let Some(subpass) = Subpass::from(render_pass, id) {
            Some(PipelineRenderPassType::BeginRenderPass(subpass))
        }
        else {
            None
        }
    }


    #[inline]
    pub fn pipeline_begin_rendering_type(
        &self,
        view_mask: u32,
        color_attachment_formats: Vec<Option<Format>>,
        depth_attachment_format: Option<Format>,
        stencil_attachment_format: Option<Format>
    ) -> PipelineRenderPassType {
        PipelineRenderPassType::BeginRendering(
            PipelineRenderingCreateInfo { 
                view_mask, 
                color_attachment_formats, 
                depth_attachment_format, 
                stencil_attachment_format, 
                ..Default::default() 
            }
        )
    }
}

unsafe impl Send for Renderer { }
unsafe impl Sync for Renderer { }



#[inline]
pub fn load_from_spv_file(
    path: &Path,
    render_ctx: &Arc<RenderContext>, 
) -> Result<Arc<ShaderModule>, RuntimeError> {
    // open file.
    let mut file = fs::File::open(path)
        .map_err(|e| err!("Failed to open file: {}", e.to_string()))?;

    // read file.
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| err!("Failed to read file: {}", e.to_string()))?;

    // create shader module.
    unsafe { ShaderModule::from_bytes(
        render_ctx.ref_device().clone(), 
        &buf
    )}.map_err(|e| err!("Shader module creation failed: {}", e.to_string()))
}
