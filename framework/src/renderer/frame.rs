use std::fmt;
use std::sync::{Arc, Mutex};

use vulkano::command_buffer::{PrimaryAutoCommandBuffer, RenderPassBeginInfo};
use vulkano::command_buffer::allocator::CommandBufferAlloc;
use vulkano::format::Format;
use vulkano::image::{SampleCount, ImageLayout};
use vulkano::render_pass::{Framebuffer, RenderPass, RenderPassCreateInfo, AttachmentDescription, LoadOp, StoreOp, SubpassDescription, AttachmentReference, SubpassDependency, FramebufferCreateInfo};
use vulkano::swapchain::{SwapchainAcquireFuture, SwapchainPresentInfo};
use vulkano::sync::{now, GpuFuture, PipelineStages, AccessFlags, FlushError}; 

use super::context::RenderContext;
use super::swapchain::RenderSwapchain;
use super::depth_stencil::RenderDepthStencil;
use crate::{err, error::RuntimeError};


pub struct RenderFrame {
    recreate_swapchain: bool,
    swapchain: RenderSwapchain,
    depth_stencil: RenderDepthStencil,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl RenderFrame {
    /// Create a new `RenderFrame`.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if Vulkan swapchain creation fails.
    /// - Returns a runtime error message if Vulkan image view creation fails.
    /// - Returns a runtime error message if there is no format supported by the device.
    /// - Returns a runtime error message if depth-stencil image creation fails.
    /// - Returns a runtime error message if depth-stencil image view creation fails.
    /// - Returns a runtime error message if render pass creation fails.
    /// - Returns a runtime error message if framebuffer creation fails.
    /// 
    pub fn new(
        width: u32,
        height: u32,
        render_ctx: &Arc<RenderContext>,
    ) -> Result<Arc<Mutex<Self>>, RuntimeError> {
        // create a `RenderSwapchain`.
        let swapchain = RenderSwapchain::new(
            width, 
            height,
            render_ctx.clone()
        )?;

        // create a `RenderDepthStencil`
        let depth_stencil = RenderDepthStencil::new(
            width, 
            height, 
            render_ctx.clone()
        )?;

        // create a vulkan render pass.
        let render_pass = create_vulkan_render_pass(
            &render_ctx,
            swapchain.ref_swapchain().image_format(), 
            depth_stencil.ref_format().clone()
        )?;

        // create a vulkan framebuffers.
        let image_extent = swapchain.ref_swapchain().image_extent();
        let framebuffers = create_vulkan_framebuffers(
            image_extent[0], 
            image_extent[1], 
            &swapchain, 
            &depth_stencil, 
            &render_pass
        )?;

        // create a waiting future.
        let previous_frame_end = Some(now(render_ctx.ref_device().clone()).boxed());

        Ok(Arc::new(Mutex::new(Self {
            recreate_swapchain: false,
            swapchain,
            depth_stencil,
            render_pass,
            framebuffers,
            previous_frame_end
        })))
    }

    
    /// Wait until the current frame image is finished drawing, then get the next frame image.
    /// 
    /// # Results
    /// - Returns `SwapchainAcquireFuture` if the next frame image is fetched successfully.
    /// - Returns `None` if `AcquireError::OutOfDate` occurs.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if getting the next frame image fails.
    /// - Returns a runtime error message if Vulkan swapchain recreation fails.
    /// - Returns a runtime error message if Vulkan image view creation fails.
    /// - Returns a runtime error message if depth-stencil image creation fails.
    /// - Returns a runtime error message if depth-stencil image view creation fails.
    /// - Returns a runtime error message if framebuffer creation fails.
    /// 
    pub fn wait_for_next_frame(
        &mut self,
        scale: f32,
        width: u32,
        height: u32
    ) -> Result<Option<(SwapchainAcquireFuture, Arc<Framebuffer>)>, RuntimeError> {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            // recreate a swapchain.
            self.swapchain.recreate(width, height)?;

            // recreate a depth-stencil.
            self.depth_stencil.recreate(width, height)?;

            // recreate a framebuffers
            self.framebuffers = create_vulkan_framebuffers(
                width, 
                height, 
                &self.swapchain, 
                &self.depth_stencil, 
                &self.render_pass
            )?;

            self.recreate_swapchain = false;

            #[cfg(feature = "monitor")]
            println!("<monitor> swapchain recreated. ({:?}, {:?})", &width, &height);
        }

        if let Some((image_index, suboptimal, acquire_future)) = self.swapchain.acquire_next_image()? {
            self.recreate_swapchain = suboptimal;
            return Ok(Some((acquire_future, self.framebuffers[image_index as usize].clone())));
        }
        else {
            return Ok(None);
        }
    }

    /// Submit commands to the queue and print them to the screen.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if command buffer execution fails.
    /// - Returns a runtime error message if presentation fails.
    /// 
    pub fn queue_submit_and_present<A: CommandBufferAlloc>(
        &mut self,
        render_ctx: &Arc<RenderContext>,
        acquire_future: SwapchainAcquireFuture,
        command_buffer: PrimaryAutoCommandBuffer<A>
    ) -> Result<(), RuntimeError> {
        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(
                render_ctx.ref_integrated_queue().clone(), 
                command_buffer
            ).map_err(|e| err!("Command buffer execution failed: {}", e.to_string()))?
            .then_swapchain_present(
                render_ctx.ref_integrated_queue().clone(), 
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.ref_swapchain().clone(), 
                    self.swapchain.get_current_frame()
                )
            ).then_signal_fence_and_flush();
        
        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            },
            Err(FlushError::OutOfDate) => {
                #[cfg(debug_assertions)]
                println!("flush error! (out of date)");

                self.recreate_swapchain = true;
                self.previous_frame_end = Some(now(render_ctx.ref_device().clone()).boxed());
            },
            Err(e) => {
                return Err(err!("Presentation failed: {}", e.to_string()));
            }
        };

        Ok(())
    }

    #[inline]
    pub fn ref_current_framebuffer(&self) -> &Arc<Framebuffer> {
        &self.framebuffers[self.swapchain.get_current_frame() as usize]
    }

    #[inline]
    pub fn ref_render_pass(&self) -> &Arc<RenderPass> {
        &self.render_pass
    }
}


impl fmt::Debug for RenderFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderFrame")
            .field("recreate_swapchain", &self.recreate_swapchain)
            .field("swapchain", &self.swapchain)
            .field("depth_stencil", &self.depth_stencil)
            .field("render_pass", &self.render_pass)
            .field("framebuffers", &self.framebuffers)
            .finish()
    }
}


/// Create a vulkan render pass.
/// 
/// # Runtime Errors 
/// - Returns a runtime error message if render pass creation fails.
/// 
#[inline]
fn create_vulkan_render_pass(
    render_ctx: &Arc<RenderContext>,
    swapchain_format: Format,
    depth_stencil_format: Format,
) -> Result<Arc<RenderPass>, RuntimeError> {
    RenderPass::new(
        render_ctx.ref_device().clone(), 
        RenderPassCreateInfo {
            attachments: vec![
                AttachmentDescription {
                    format: Some(swapchain_format),
                    samples: SampleCount::Sample1,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    stencil_load_op: LoadOp::DontCare,
                    stencil_store_op: StoreOp::DontCare,
                    initial_layout: ImageLayout::Undefined,
                    final_layout: ImageLayout::PresentSrc,
                    ..Default::default()
                },
                AttachmentDescription {
                    format: Some(depth_stencil_format),
                    samples: SampleCount::Sample1,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    stencil_load_op: LoadOp::Clear,
                    stencil_store_op: StoreOp::DontCare,
                    initial_layout: ImageLayout::Undefined,
                    final_layout: ImageLayout::DepthStencilAttachmentOptimal,
                    ..Default::default()
                }
            ],
            subpasses: vec![
                SubpassDescription {
                    color_attachments: vec![
                        Some(AttachmentReference {
                            attachment: 0,
                            layout: ImageLayout::ColorAttachmentOptimal,
                            ..Default::default()
                        })
                    ],
                    depth_stencil_attachment: Some(
                        AttachmentReference {
                            attachment: 1,
                            layout: ImageLayout::DepthStencilAttachmentOptimal,
                            ..Default::default()
                        }
                    ),
                    ..Default::default()
                }
            ],
            dependencies: vec![
                SubpassDependency {
                    src_subpass: None,
                    dst_subpass: Some(0),
                    src_stages: PipelineStages::EARLY_FRAGMENT_TESTS | PipelineStages::LATE_FRAGMENT_TESTS,
                    dst_stages: PipelineStages::EARLY_FRAGMENT_TESTS | PipelineStages::LATE_FRAGMENT_TESTS,
                    src_access: AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    dst_access: AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    ..Default::default()
                },
                SubpassDependency {
                    src_subpass: None,
                    dst_subpass: Some(0),
                    src_stages: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
                    dst_stages: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
                    src_access: AccessFlags::default(),
                    dst_access: AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE,
                    ..Default::default()
                }
            ],
            ..Default::default()
        }
    ).map_err(|e| err!("Vulkan render pass creation failed: {}", e.to_string()))
}


/// Create a framebuffers.
/// 
/// # Runtime Errors 
/// - Returns a runtime error message if framebuffer creation fails.
/// 
#[inline]
fn create_vulkan_framebuffers(
    width: u32,
    height: u32,
    swapchain: &RenderSwapchain,
    depth_stencil: &RenderDepthStencil,
    render_pass: &Arc<RenderPass>
) -> Result<Vec<Arc<Framebuffer>>, RuntimeError> {
    let mut framebuffers = Vec::with_capacity(swapchain.get_max_frame_in_flight() as usize);
    for view in swapchain.ref_swapchain_image_views().iter() {
        framebuffers.push(
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![
                        view.clone(),
                        depth_stencil.ref_image_view().clone()
                    ],
                    extent: [width, height],
                    layers: 1,
                    ..Default::default()
                }
            ).map_err(|e| err!("Framebuffer creation failed: {}", e.to_string()))?
        );
    }
    return Ok(framebuffers);
}
