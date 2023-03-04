use std::sync::Arc;
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceExtensions, InstanceCreateInfo};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo};
use vulkano::swapchain::{Surface, PresentMode, ColorSpace, CompositeAlpha, Swapchain, SwapchainCreateInfo};
use vulkano::image::{ImageAccess, ImageAspects, ImageUsage, ImageSubresourceRange, ImageLayout, SampleCount, SwapchainImage, AttachmentImage};
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::render_pass::{RenderPass, RenderPassCreateInfo, AttachmentDescription, AttachmentReference, SubpassDependency, SubpassDescription, LoadOp, StoreOp, Framebuffer, FramebufferCreateInfo};
use vulkano::format::{Format, FormatFeatures};
use vulkano::sync::{PipelineStages, AccessFlags};
use vulkano::memory::allocator::MemoryAllocator;
use crate::{err, error::RuntimeError};

#[inline]
pub fn rgb(red: u8, green: u8, blue: u8) -> (f32, f32, f32) {
    (red as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0)
}

#[inline]
pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> (f32, f32, f32, f32) {
    (red as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0, alpha as f32 / 255.0)
}

#[inline]
pub fn get_instance_extensions() -> InstanceExtensions {
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
pub fn load_vulkan_library() -> Result<Arc<VulkanLibrary>, RuntimeError> {
    return VulkanLibrary::new()
        .map_err(|e| err!("Vk Load Error:{}", e.to_string()));
}

#[inline]
pub fn create_vulkan_instance(library: &Arc<VulkanLibrary>) 
-> Result<Arc<Instance>, RuntimeError> {
    let enabled_extensions = library.supported_extensions()
        .intersection(&get_instance_extensions());

    return Instance::new(
        library.clone(), 
        InstanceCreateInfo {
            enabled_extensions,
            enumerate_portability: true,
            ..Default::default()
        }
    ).map_err(|e| err!("Vk Create Error:{}", e.to_string()));
}

#[inline]
pub fn create_vulkan_device_and_queue(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>
) -> Result<(Arc<Device>, Arc<Queue>), RuntimeError> {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    };

    let device_features = Features::default();

    let (physical_device, queue_family_index) = match instance
        .enumerate_physical_devices()
        .map_err(|e| err!("Vk Device Query Error:{}", e.to_string()))?
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
            None => return Err(err!("No suitable physical device found."))
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
        .map_err(|e| err!("Vk Create Error:{}", e.to_string()))?;

    Ok((device, queues.next().unwrap()))
}

#[inline]
pub fn create_vulkan_swapchain(
    screen_size: (u32, u32),
    device: &Arc<Device>,
    surface: &Arc<Surface>
) -> Result<(Arc<Swapchain>, Vec<Arc<ImageView<SwapchainImage>>>), RuntimeError> {
    let surface_capacities = device
        .physical_device()
        .surface_capabilities(surface, Default::default())
        .map_err(|e| err!("Vk Device Query Error:{}", e.to_string()))?;

    let present_mode = device
        .physical_device()
        .surface_present_modes(&surface)
        .map_err(|e| err!("Vk Device Query Error:{}", e.to_string()))?
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
        .map_err(|e| err!("Vk Device Query Error:{}", e.to_string()))?;
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
            image_extent: [screen_size.0, screen_size.1],
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
    ).map_err(|e| err!("Vk Create Error:{}", e.to_string()))?;

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
            ).map_err(|e| err!("Vk Create Error:{}", e.to_string()))
        }).collect::<Result<_, RuntimeError>>()?;

    Ok((swapchain, swapchain_image_view))
}

#[inline]
pub fn get_depth_stencil_format(device: &Arc<Device>) 
-> Result<Format, RuntimeError> {
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
        None => return Err(err!("No suitable depth-stencil format found."))
    };
}

#[inline]
pub fn create_depth_stencil(
    screen_size: (u32, u32),
    allocator: &impl MemoryAllocator,
    device: &Arc<Device>,
) -> Result<Arc<ImageView<AttachmentImage>>, RuntimeError> {
    let depth_stencil_format = get_depth_stencil_format(device)?;

    let depth_stencil_image = AttachmentImage::with_usage(
        allocator, 
        [screen_size.0, screen_size.1], 
        depth_stencil_format, 
        ImageUsage { depth_stencil_attachment: true, ..Default::default() }
    ).map_err(|e| err!("Vk Create Error:{}", e.to_string()))?;

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
    ).map_err(|e| err!("Vk Create Error:{}", e.to_string()));
}

#[inline]
pub fn create_vulkan_render_pass(
    device: &Arc<Device>,
    swapchain_format: Option<Format>,
    depth_stencil_format: Option<Format>
) -> Result<Arc<RenderPass>, RuntimeError> {
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
    ).map_err(|e| err!("Vk Create Error:{}", e.to_string()));
}

#[inline]
pub fn create_vulkan_framebuffers(
    screen_size: (u32, u32),
    render_pass: &Arc<RenderPass>,
    swapchain_image_views: &Vec<Arc<ImageView<SwapchainImage>>>,
    depth_stencil_image_view: &Arc<ImageView<AttachmentImage>>,
) -> Result<Vec<Arc<Framebuffer>>, RuntimeError> {
    let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());
    for swapchain_image_view in swapchain_image_views.iter() {
        framebuffers.push(Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![
                    swapchain_image_view.clone(),
                    depth_stencil_image_view.clone()
                ],
                extent: [screen_size.0, screen_size.1],
                layers: 1,
                ..Default::default()
            })
            .map_err(|e| err!("Vk Create Error:{}", e.to_string()))?
        );
    }
    return Ok(framebuffers);
}
