#![allow(unused_imports)]

use std::sync::Arc;
use std::ffi::c_void;

use vulkano::VulkanLibrary;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents, PrimaryAutoCommandBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::memory::allocator::{StandardMemoryAllocator, MemoryAllocator, FastMemoryAllocator};
use vulkano::sync::{self, PipelineStages, AccessFlags, GpuFuture, FlushError};
use vulkano::format::{Format, FormatFeatures, ClearValue};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{DeviceExtensions, Device, DeviceCreateInfo, QueueCreateInfo, Features, Queue};
use vulkano::render_pass::{RenderPass, RenderPassCreateInfo, AttachmentDescription, LoadOp, StoreOp, SubpassDependency, SubpassDescription, AttachmentReference, Framebuffer, FramebufferCreateInfo};
use vulkano::swapchain::{Surface, ColorSpace, Swapchain, SwapchainCreateInfo, PresentMode, CompositeAlpha, acquire_next_image, AcquireError, SwapchainPresentInfo};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::image::{ImageUsage, SwapchainImage, ImageLayout, SampleCount, ImageViewType, ImageSubresourceRange, ImageAspects, ImageViewAbstract, AttachmentImage, ImageAccess};

#[cfg(target_os = "ios")]
use vulkano::swapchain::IOSMetalLayer;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use objc::{class, msg_send, sel, sel_impl, runtime::Object};

use crate::framework::Extent2D;

#[inline]
fn load_vulkan_library() -> Result<Arc<VulkanLibrary>, String> {
    return VulkanLibrary::new().map_err(|e| e.to_string());
}

#[inline]
fn get_instance_extensions() -> InstanceExtensions {
    return InstanceExtensions {
        khr_surface: true,
        khr_android_surface: true,
        khr_xcb_surface: true,
        khr_xlib_surface: true,
        khr_wayland_surface: true,
        mvk_ios_surface: true,
        mvk_macos_surface: true,
        khr_win32_surface: true,
        khr_get_physical_device_properties2: true,
        khr_get_surface_capabilities2: true,
        ..Default::default()
    };
}

#[inline]
fn create_vulkan_instance(library: &Arc<VulkanLibrary>) 
-> Result<Arc<Instance>, String> {
    let enabled_extensions = library.supported_extensions()
        .intersection(&get_instance_extensions());

    return Instance::new(
        library.clone(), 
        InstanceCreateInfo {
            enabled_extensions,
            enumerate_portability: true,
            ..Default::default()
        }
    ).map_err(|e| e.to_string());
}

#[inline]
#[cfg(target_os = "ios")]
fn create_vulkan_surface_ios(view: *mut c_void, instance: &Arc<Instance>) 
-> Result<Arc<Surface>, String> {
    unsafe {
        let view: *mut Object = std::mem::transmute(view);
        let main_layer: *mut Object = msg_send![view, layer];
        return  Surface::from_ios(
            instance.clone(), 
            IOSMetalLayer::new(view, main_layer),
            None)
            .map_err(|e| e.to_string());
    }
}

#[inline]
fn create_vulkan_device_and_queue(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>
) -> Result<(Arc<Device>, Arc<Queue>), String> {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    };

    let device_features = Features::default();

    let (physical_device, queue_family_index) = match instance
        .enumerate_physical_devices()
        .map_err(|e| e.to_string())?
        .filter(|physical_device| {
            physical_device.supported_extensions().contains(&device_extensions)
            && physical_device.supported_features().contains(&device_features)
        })
        .filter_map(|physical_device| {
            physical_device.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(idx, properties)| {
                    properties.queue_flags.graphics 
                    && physical_device.surface_support(idx as u32, surface).unwrap_or(false)
                })
                .map(|idx| { (physical_device, idx as u32) })
        })
        .min_by_key(|(physical_device, _)| {
            match physical_device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            }
        }) {
            Some(item) => item,
            None => return Err("No suitable physical device found.".to_string())
        };

    let (device, mut queues) = Device::new(
        physical_device, 
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        })
        .map_err(|e| e.to_string())?;

    Ok((device, queues.next().unwrap()))
}

#[inline]
fn create_vulkan_swapchain(
    screen_size: Extent2D<u32>,
    device: &Arc<Device>,
    surface: &Arc<Surface>
) -> Result<(Arc<Swapchain>, Vec<Arc<ImageView<SwapchainImage>>>), String> {
    let surface_capacities = device
        .physical_device()
        .surface_capabilities(surface, Default::default())
        .map_err(|e| e.to_string())?;

    let present_mode = device
        .physical_device()
        .surface_present_modes(&surface)
        .map_err(|e| e.to_string())?
        .min_by_key(|&mode| {
            match mode {
                PresentMode::Mailbox => 1,
                PresentMode::Immediate => 2,
                PresentMode::FifoRelaxed => 3,
                PresentMode::Fifo => 4,
                _ => 5,
            }
        }).unwrap_or(PresentMode::Fifo);
    
    let surface_formats = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .map_err(|e| e.to_string())?;
    let (image_format, image_color_space) = surface_formats
        .iter()
        .find(|&&(format, color_space)| {   
            format == Format::B8G8R8A8_SNORM && color_space == ColorSpace::SrgbNonLinear
        }).map(|&(format, color_space)| {
            (Some(format), color_space)
        }).unwrap_or((Some(surface_formats[0].0), surface_formats[0].1));

    let (swapchain, swapchain_images) = Swapchain::new(
        device.clone(), 
        surface.clone(), 
        SwapchainCreateInfo {
            min_image_count: surface_capacities.min_image_count,
            image_format,
            image_color_space,
            image_extent: [screen_size.width, screen_size.height],
            image_array_layers: 1,
            image_usage: ImageUsage { 
                color_attachment: true,
                ..Default::default()
            },
            pre_transform: surface_capacities.current_transform,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode,
            clipped: true,
            ..Default::default()
        }
    ).map_err(|e| e.to_string())?;

    let swapchain_image_view = swapchain_images
        .iter()
        .map(|image| {
            ImageView::new(
                image.clone(),
                ImageViewCreateInfo {
                    view_type: ImageViewType::Dim2d,
                    format: Some(image.format()),
                    subresource_range: ImageSubresourceRange {
                        mip_levels: (0..1),
                        array_layers: (0..1),
                        aspects: ImageAspects {
                            color: true, ..Default::default()
                        }
                    },
                    ..Default::default()
                }
            ).map_err(|e| e.to_string())
        }).collect::<Result<_, String>>()?;

    Ok((swapchain, swapchain_image_view))
}

#[inline]
fn get_depth_stencil_format(device: &Arc<Device>) -> Result<Format, String> {
    const CANDIDATE_FORMATS: [Format; 3] = [
        Format::D32_SFLOAT_S8_UINT,
        Format::D24_UNORM_S8_UINT,
        Format::D16_UNORM_S8_UINT
    ];

    return match CANDIDATE_FORMATS.iter()
        .filter_map(|&format| {
            match device.physical_device().format_properties(format) {
                Ok(properties) => Some((format, properties)),
                _ => None
            }
        })
        .find_map(|(format, properties)| {
            if properties.optimal_tiling_features.contains(&FormatFeatures {
                depth_stencil_attachment: true,
                ..Default::default()
            }) {
                Some(format)
            }
            else {
                None
            }
        }) {
        Some(format) => Ok(format),
        None => return Err("No suitable depth-stencil format found.".to_string())
    };
}

#[inline]
fn create_depth_stencil(
    screen_size: Extent2D<u32>,
    allocator: &impl MemoryAllocator,
    device: &Arc<Device>,
) -> Result<Arc<ImageView<AttachmentImage>>, String> {
    let depth_stencil_format = get_depth_stencil_format(device)?;

    let depth_stencil_image = AttachmentImage::with_usage(
        allocator, 
        [screen_size.width, screen_size.height], 
        depth_stencil_format, 
        ImageUsage { depth_stencil_attachment: true, ..Default::default() }
    ).map_err(|e| e.to_string())?;

    return ImageView::new(
        depth_stencil_image,
        ImageViewCreateInfo {
            view_type: ImageViewType::Dim2d,
            format: Some(depth_stencil_format),
            subresource_range: ImageSubresourceRange {
                mip_levels: (0..1),
                array_layers: (0..1),
                aspects: ImageAspects {
                    depth: true, stencil: true,
                    ..Default::default()
                }
            },
            ..Default::default()
        }
    ).map_err(|e| e.to_string());
}

#[inline]
fn create_vulkan_render_pass(
    device: &Arc<Device>,
    swapchain_format: Option<Format>,
    depth_stencil_format: Option<Format>
) -> Result<Arc<RenderPass>, String> {
    return RenderPass::new(
        device.clone(), 
        RenderPassCreateInfo {
            attachments: vec![
                AttachmentDescription {
                    format: swapchain_format,
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
                    format: depth_stencil_format,
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
            dependencies: vec![
                SubpassDependency {
                    src_subpass: None,
                    dst_subpass: Some(0),
                    src_stages: PipelineStages {
                        early_fragment_tests: true,
                        late_fragment_tests: true,
                        ..Default::default()
                    },
                    dst_stages: PipelineStages {
                        early_fragment_tests: true,
                        late_fragment_tests: true,
                        ..Default::default()
                    },
                    src_access: AccessFlags {
                        depth_stencil_attachment_write: true,
                        ..Default::default()
                    },
                    dst_access: AccessFlags {
                        depth_stencil_attachment_read: true,
                        depth_stencil_attachment_write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                SubpassDependency {
                    src_subpass: None,
                    dst_subpass: Some(0),
                    src_stages: PipelineStages {
                        color_attachment_output: true,
                        ..Default::default()
                    },
                    dst_stages: PipelineStages {
                        color_attachment_output: true,
                        ..Default::default()
                    },
                    src_access: AccessFlags::default(),
                    dst_access: AccessFlags {
                        color_attachment_read: true,
                        color_attachment_write: true,
                        ..Default::default()
                    },
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
            ..Default::default()
        }
    ).map_err(|e| e.to_string());
}

#[inline]
fn create_vulkan_framebuffers(
    screen_size: Extent2D<u32>,
    render_pass: &Arc<RenderPass>,
    swapchain_image_views: &Vec<Arc<ImageView<SwapchainImage>>>,
    depth_stencil_image_view: &Arc<ImageView<AttachmentImage>>,
) -> Result<Vec<Arc<Framebuffer>>, String> {
    let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());
    for swapchain_image_view in swapchain_image_views.iter() {
        framebuffers.push(Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![
                    swapchain_image_view.clone(),
                    depth_stencil_image_view.clone()
                ],
                extent: [screen_size.width, screen_size.height],
                layers: 1,
                ..Default::default()
            })
            .map_err(|e| e.to_string())?
        );
    }
    return Ok(framebuffers);
}

pub struct Renderer {
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
    #[cfg(target_os = "ios")]
    pub fn new_ios_ver(
        view: *mut c_void,
        scale: f32,
        screen_size: Extent2D<u32>,
    ) -> Result<Self, String> {
        let library = load_vulkan_library()?;
        let instance = create_vulkan_instance(&library)?;
        let surface = create_vulkan_surface_ios(view, &instance)?;
        let (device, queue) = create_vulkan_device_and_queue(&instance, &surface)?;
        let (swapchain, swapchain_image_views) = create_vulkan_swapchain(screen_size, &device, &surface)?;
        let std_mem_allocator = StandardMemoryAllocator::new_default(device.clone());
        let fast_mem_allocator = FastMemoryAllocator::new_default(device.clone());
        let depth_stencil_image_view = create_depth_stencil(screen_size, &std_mem_allocator, &device)?;
        let render_pass = create_vulkan_render_pass(&device, Some(swapchain.image_format()), depth_stencil_image_view.format())?;
        let framebuffers = create_vulkan_framebuffers(screen_size, &render_pass, &swapchain_image_views, &depth_stencil_image_view)?;
        let cmd_buf_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        Ok(Self { 
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

    #[inline(always)]
    pub fn get_std_memory_allocator_ref(&self) -> &StandardMemoryAllocator {
        &self.std_mem_allocator
    }

    #[inline(always)]
    pub fn get_fast_memory_allocator_ref(&self) -> &FastMemoryAllocator {
        &self.fast_mem_allocator
    }

    #[inline(always)]
    pub fn get_cmd_buf_allocator_ref(&self) -> &StandardCommandBufferAllocator {
        &self.cmd_buf_allocator
    }

    pub fn draw(&mut self, red: f32, green: f32, blue: f32) -> Result<(), String> {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();
        let (img_idx, suboptimal, acquire_future) = 
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(item) => item,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                },
                Err(e) => {
                    return Err(e.to_string());
                }
        };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.cmd_buf_allocator, 
            self.queue.queue_family_index(), 
            CommandBufferUsage::OneTimeSubmit)
            .map_err(|e| e.to_string())?;

        builder.begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some(ClearValue::Float([red, green, blue, 1.0])),
                    Some(ClearValue::DepthStencil((1.0, 0)))
                ],
                ..RenderPassBeginInfo::framebuffer(
                    self.framebuffers[img_idx as usize].clone()
                )
            }, 
            SubpassContents::Inline,
        ).map_err(|e| e.to_string())?;

        /* ...rendering code... */

        builder.end_render_pass()
            .map_err(|e| e.to_string())?;

        let command_buffer = builder.build().map_err(|e| e.to_string())?;
        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .map_err(|e| e.to_string())?
            .then_swapchain_present(
                self.queue.clone(), 
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(), 
                    img_idx
                )
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            },
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            },
            Err(e) => {
                return Err(e.to_string())
            }
        }

        Ok(())
    }
}