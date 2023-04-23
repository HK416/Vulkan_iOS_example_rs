use std::sync::Arc;

use vulkano::format::Format;
use vulkano::sampler::ComponentMapping;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::{SwapchainImage, ImageUsage, ImageViewType, ImageSubresourceRange, ImageAspects};
use vulkano::swapchain::{self, Swapchain, SwapchainCreateInfo, SwapchainAcquireFuture, AcquireError, PresentMode, ColorSpace, CompositeAlpha};
use vulkano::sync::Sharing;

use super::context::RenderContext;
use crate::{err, error::RuntimeError};



#[derive(Debug)]
pub struct RenderSwapchain {
    current_frame: u32,
    max_frame_in_flight: u32,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<SwapchainImage>>,
    views: Vec<Arc<ImageView<SwapchainImage>>>,
    render_ctx: Arc<RenderContext>,
}

impl RenderSwapchain {
    /// Create a new `RenderSwapchain`.
    /// 
    /// ### Note
    /// - If there is an existing swap chain, do not create a new swap chain by calling this function.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if Vulkan swapchain creation fails.
    /// - Returns a runtime error message if Vulkan image view creation fails.
    /// 
    pub fn new(
        width: u32, 
        height: u32, 
        render_ctx: Arc<RenderContext>
    ) -> Result<Self, RuntimeError> {
        let (max_frame_in_flight, swapchain, images, views) 
            = create_vulkan_swapchain(width, height, &render_ctx)?;

        Ok(Self {
            current_frame: 0,
            max_frame_in_flight,
            render_ctx,
            swapchain,
            images,
            views,
        })
    }


    /// Create a new swapchain based on the existing swapchain.
    /// 
    /// # Runtime Errors
    /// 
    /// - Returns a runtime error message if Vulkan swapchain recreation fails.
    /// - Returns a runtime error message if Vulkan image view creation fails.
    /// 
    pub fn recreate(&mut self, width: u32, height: u32) -> Result<(), RuntimeError> {
        let surface_capabilities = self.render_ctx.get_surface_capabilities()?;
        let image_extent = surface_capabilities.current_extent.unwrap_or([width, height]);

        // recreate a swapchain and swapchain images.
        let (swapchain, images) = self.swapchain.recreate(
            SwapchainCreateInfo {
                image_extent,
                ..self.swapchain.create_info()
            }
        ).map_err(|e| err!("Swapchain recreation failed: {}", e.to_string()))?;

        let views = create_vulkan_swapchain_image_views(
            Some(swapchain.image_format()), &images
        )?;

        self.current_frame = 0;
        self.swapchain = swapchain;
        self.images = images;
        self.views = views;

        Ok(())
    }


    /// Get the next frame image.
    /// 
    /// ## Results
    /// - Returns `None` if `AcquireError::OutOfDate` occurs.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if getting the next frame image fails.
    /// 
    pub fn acquire_next_image(&mut self) -> Result<Option<(u32, bool, SwapchainAcquireFuture)>, RuntimeError> {
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(it) => it,
                Err(AcquireError::OutOfDate) => {
                    return Ok(None);
                },
                Err(e) => {
                    return Err(err!("Failed to get swapchain next image: {}", e.to_string()))
                }
            };

        self.current_frame = image_index;
        Ok(Some((image_index, suboptimal, acquire_future)))
    }


    /// Get the current swapchain image index.
    #[inline]
    pub fn get_current_frame(&self) -> u32 {
        self.current_frame
    }

    /// Get the number of swapchain images.
    #[inline]
    pub fn get_max_frame_in_flight(&self) -> u32 {
        self.max_frame_in_flight
    }


    /// Get the vulkan swapchain. (reference)
    #[inline]
    pub fn ref_swapchain(&self) -> &Arc<Swapchain> {
        &self.swapchain
    }


    /// Get the vulkan swapchain images. (reference)
    #[inline]
    pub fn ref_swapchain_images(&self) -> &[Arc<SwapchainImage>] {
        &self.images
    }


    /// Get the vulkan swapchain image views. (reference)
    #[inline]
    pub fn ref_swapchain_image_views(&self) -> &[Arc<ImageView<SwapchainImage>>] {
        &self.views
    }
}


/// Create a vulkan swapchain, swapchain images, and image views.
/// 
/// # Runtime Errors
/// - Returns a runtime error message if Vulkan swapchain creation fails.
/// - Returns a runtime error message if Vulkan image view creation fails.
/// 
#[inline]
fn create_vulkan_swapchain(
    width: u32,
    height: u32,
    render_ctx: &RenderContext
) -> Result<(u32, Arc<Swapchain>, Vec<Arc<SwapchainImage>>, Vec<Arc<ImageView<SwapchainImage>>>), RuntimeError> {
    let surface_capabilities = render_ctx.get_surface_capabilities()?;
    let image_extent = surface_capabilities.current_extent.unwrap_or([width, height]);

    // set the present mode. (default = `PresentMode::Fifo`)
    let present_mode = render_ctx
        .get_surface_present_modes()?
        .min_by_key(|&mode| {
            match mode {
                PresentMode::Mailbox => 1,
                PresentMode::Immediate => 2,
                PresentMode::FifoRelaxed => 3,
                PresentMode::Fifo => 4,
                _ => 5,
            }
        })
        .unwrap_or(PresentMode::Fifo);

    // finds surfaces of a specific type.
    // if not found, the device's default settings are used.
    let (image_format, image_color_space) = render_ctx
        .get_surface_formats()?
        .into_iter()
        .find(|(format, color_space)| {
            format.clone() == Format::B8G8R8A8_UNORM 
            && color_space.clone() == ColorSpace::SrgbNonLinear
        })
        .unzip();
    
    // set the number of swap chain buffers.
    //
    // Note: Triple buffering is recommended on macOS/iOS.
    // MoltenVk Guide: <https://github.com/KhronosGroup/MoltenVK/blob/main/Docs/MoltenVK_Runtime_UserGuide.md>
    //
    let max_frame_in_flight = 3.clamp(
        surface_capabilities.min_image_count, 
        surface_capabilities.max_image_count.unwrap_or(surface_capabilities.min_image_count)
    );

    // set the image usage flags.
    let mut image_usage = ImageUsage::COLOR_ATTACHMENT;
    if surface_capabilities.supported_usage_flags.contains(ImageUsage::TRANSFER_SRC) {
        image_usage |= ImageUsage::TRANSFER_SRC;
    }
    if surface_capabilities.supported_usage_flags.contains(ImageUsage::TRANSFER_DST) {
        image_usage |= ImageUsage::TRANSFER_DST;
    }

    // create a swapchain and swapchain images.
    let (swapchain, images) = Swapchain::new(
        render_ctx.ref_device().clone(), 
        render_ctx.ref_surface().clone(), 
        SwapchainCreateInfo {
            min_image_count: max_frame_in_flight,
            image_format,
            image_color_space: image_color_space.unwrap_or(ColorSpace::SrgbNonLinear),
            image_extent,
            image_array_layers: 1,
            image_usage,
            image_sharing: Sharing::Exclusive,
            pre_transform: surface_capabilities.current_transform,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode,
            clipped: true,
            ..Default::default()
        }
    ).map_err(|e| err!("Swapchain creation failed: {}", e.to_string()))?;

    // create a image views from swapchain images.
    let views = create_vulkan_swapchain_image_views(
        image_format, 
        &images
    )?;
    
    Ok((max_frame_in_flight, swapchain, images, views))
}


/// Create a image views from swapchain images.
/// 
/// # Runtime Errors
/// - Returns a runtime error message if Vulkan image view creation fails.
///
/// # Panics
/// - Stop program execution if the swapchain image is empty.
/// 
#[inline]
fn create_vulkan_swapchain_image_views(
    format: Option<Format>,
    images: &[Arc<SwapchainImage>]
) -> Result<Vec<Arc<ImageView<SwapchainImage>>>, RuntimeError> {
    assert!(!images.is_empty(), "At least one swap chain image must exist.");

    images.iter()
        .map(|image| {
            ImageView::new(
                image.clone(),
                ImageViewCreateInfo {
                    view_type: ImageViewType::Dim2d,
                    format,
                    component_mapping: ComponentMapping::identity(),
                    subresource_range: ImageSubresourceRange {
                        aspects: ImageAspects::COLOR,
                        mip_levels: (0..1),
                        array_layers: (0..1)
                    },
                    ..Default::default()
                }
            ).map_err(|e| err!("Swapchain image view creation failed: {}", e.to_string()))
        }
    ).collect::<Result<_, RuntimeError>>()
}
