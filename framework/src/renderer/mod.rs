mod utility;
mod platform;

pub use std::sync::Arc;
pub use vulkano::VulkanLibrary;
pub use vulkano::instance::{Instance, InstanceExtensions, InstanceCreateInfo};
pub use vulkano::device::physical::PhysicalDeviceType;
pub use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo};
pub use vulkano::swapchain::{acquire_next_image, AcquireError, SwapchainPresentInfo, SwapchainAcquireFuture, Surface, PresentMode, ColorSpace, CompositeAlpha, Swapchain, SwapchainCreateInfo};
pub use vulkano::image::{ImageAccess, ImageAspects, ImageUsage, ImageSubresourceRange, ImageLayout, SampleCount, ImageViewAbstract, SwapchainImage, AttachmentImage};
pub use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
pub use vulkano::render_pass::{RenderPass, RenderPassCreateInfo, AttachmentDescription, AttachmentReference, SubpassDependency, SubpassDescription, LoadOp, StoreOp, Framebuffer, FramebufferCreateInfo};
pub use vulkano::memory::allocator::{StandardMemoryAllocator, FastMemoryAllocator, MemoryAllocator};
pub use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer, SecondaryCommandBufferAbstract, ExecuteCommandsError,  CommandBufferUsage, RenderPassBeginInfo, SubpassContents};
pub use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
pub use vulkano::sync::{PipelineStages, AccessFlags, GpuFuture, FlushError};
pub use vulkano::format::{Format, FormatFeatures, ClearValue};
pub use crate::{err, error::RuntimeError};
pub use self::platform::AppHandle;
pub use self::utility::{rgb, rgba};

use utility::*;
use platform::*;

pub struct Renderer {
    handle: AppHandle,
    screen_size: (u32, u32),
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    swapchain_image_views: Vec<Arc<ImageView<SwapchainImage>>>,
    depth_stencil_image_view: Arc<ImageView<AttachmentImage>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    std_mem_allocator: StandardMemoryAllocator,
    fast_mem_allocator: FastMemoryAllocator,
    cmd_buf_allocator: StandardCommandBufferAllocator,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swapchain: bool,
}

impl Renderer {
    pub fn new(
        handle: AppHandle, 
        screen_size: Option<(u32, u32)>
    ) -> Result<Self, RuntimeError> {
        let screen_size =  match screen_size {
            Some(size) => size,
            None => get_screen_size(&handle)?
        };

        let library = load_vulkan_library()?;
        let instance = create_vulkan_instance(&library)?;
        let surface = create_vulkan_surface(&handle, &instance)?;
        let (device, queue) = create_vulkan_device_and_queue(&instance, &surface)?;
        let (swapchain, swapchain_image_views) = create_vulkan_swapchain(screen_size, &device, &surface)?;
        let std_mem_allocator = StandardMemoryAllocator::new_default(device.clone());
        let fast_mem_allocator = FastMemoryAllocator::new_default(device.clone());
        let depth_stencil_image_view = create_depth_stencil(screen_size, &std_mem_allocator, &device)?;
        let render_pass = create_vulkan_render_pass(&device, Some(swapchain.image_format()), depth_stencil_image_view.format())?;
        let framebuffers = create_vulkan_framebuffers(screen_size, &render_pass, &swapchain_image_views, &depth_stencil_image_view)?;
        let cmd_buf_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
        let previous_frame_end = Some(vulkano::sync::now(device.clone()).boxed());

        Ok(Self { 
            handle,
            screen_size,
            instance, 
            surface, 
            device,
            queue,
            swapchain,
            swapchain_image_views,
            depth_stencil_image_view,
            render_pass,
            framebuffers,
            std_mem_allocator,
            fast_mem_allocator,
            cmd_buf_allocator,
            previous_frame_end,
            recreate_swapchain: false,
        })
    }

    #[inline]
    pub fn prepare_render(
        &mut self, 
        red: f32, 
        green: f32, 
        blue: f32, 
        alpha: f32
    ) -> Result<Option<PresentCommandBufferGuard>, RuntimeError> {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();
        let (image_index, suboptimal, acquire_future) = 
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(item) => item,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(None);
                },
                Err(e) => {
                    return Err(err!("Vk Present Error:{}", e.to_string()));
                }
        };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.cmd_buf_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).map_err(|e| err!("Vk Present Error:{}", e.to_string()))?;

        builder.begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some(ClearValue::Float([red, green, blue, alpha])),
                    Some(ClearValue::DepthStencil((1.0, 0))),
                ],
                ..RenderPassBeginInfo::framebuffer(
                    self.framebuffers[image_index as usize].clone()
                )
            }, 
            SubpassContents::Inline
        ).map_err(|e| err!("Vk Present Error:{}", e.to_string()))?;

        return Ok(Some(PresentCommandBufferGuard { 
            image_index, 
            acquire_future, 
            builder 
        }));
    }

    #[inline]
    pub fn submit_and_present(
        &mut self,
        mut guard: PresentCommandBufferGuard,
    ) -> Result<(), RuntimeError> {
        guard.builder.end_render_pass()
            .map_err(|e| err!("Vk Present Error:{}", e.to_string()))?;

        let command_buffer = guard.builder.build().map_err(|e| err!("Vk Present Error:{}", e.to_string()))?;
        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(guard.acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .map_err(|e| err!("Vk Present Error:{}", e.to_string()))?
            .then_swapchain_present(
                self.queue.clone(), 
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(), 
                    guard.image_index
                )
            ).then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed())
            },
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(vulkano::sync::now(self.device.clone()).boxed());
            },
            Err(e) => {
                return Err(err!("Vk Present Error:{}", e.to_string()));
            }
        };

        Ok(())
    }
}

pub struct PresentCommandBufferGuard {
    image_index: u32,
    acquire_future: SwapchainAcquireFuture,
    builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
}

impl PresentCommandBufferGuard {
    #[inline]
    pub fn execute_commands<C>(&mut self, command_buffer: C) -> Result<(), ExecuteCommandsError> 
    where C: SecondaryCommandBufferAbstract + 'static {
        self.builder.execute_commands(command_buffer)?;
        Ok(())
    }

    #[inline]
    pub fn execute_commands_from_vec<C>(&mut self, command_buffers: Vec<C>) -> Result<(), ExecuteCommandsError> 
    where C: SecondaryCommandBufferAbstract + 'static {
        self.builder.execute_commands_from_vec(command_buffers)?;
        Ok(())
    }
}
