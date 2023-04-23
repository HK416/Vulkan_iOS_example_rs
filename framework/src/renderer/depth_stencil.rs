use std::sync::Arc;

use vulkano::format::{Format, FormatFeatures};
use vulkano::image::{AttachmentImage, ImageUsage, ImageViewType, ImageSubresourceRange, ImageAspects};
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::memory::allocator::MemoryAllocator;
use vulkano::sampler::ComponentMapping;

use super::context::RenderContext;
use crate::{err, error::RuntimeError};



#[derive(Debug)]
pub struct RenderDepthStencil {
    format: Format,
    image: Arc<AttachmentImage>,
    view: Arc<ImageView<AttachmentImage>>,
    render_ctx: Arc<RenderContext>,
}


impl RenderDepthStencil {
    /// Create a new `RenderDepthStencil`
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if there is no format supported by the device.
    /// - Returns a runtime error message if depth-stencil image creation fails.
    /// - Returns a runtime error message if depth-stencil image view creation fails.
    /// 
    pub fn new(
        width: u32, 
        height: u32, 
        render_ctx: Arc<RenderContext>
    ) -> Result<Self, RuntimeError> {
        if let Some(format) = get_depth_stencil_format(&render_ctx) {
            let (image, view) = create_depth_stencil(
                width, 
                height, 
                format, 
                render_ctx.ref_memory_allocator()
            )?;

            Ok(Self { format, image, view, render_ctx })
        }
        else {
            Err(err!("No suitable depth-stencil format found."))
        }
    }


    /// Create a new depth-stencil based on the existing depth-stencil.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if depth-stencil image creation fails.
    /// - Returns a runtime error message if depth-stencil image view creation fails.
    /// 
    pub fn recreate(&mut self, width: u32, height: u32) -> Result<(), RuntimeError> {
        let (image, view) = create_depth_stencil(
            width, 
            height, 
            self.format, 
            self.render_ctx.ref_memory_allocator()
        )?;

        self.image = image;
        self.view = view;

        Ok(())
    }


    /// Get the depth-stencil format. (reference)
    #[inline]
    pub fn ref_format(&self) -> &Format {
        &self.format
    }


    /// Get the depth-stencil image. (reference)
    #[inline]
    pub fn ref_image(&self) -> &Arc<AttachmentImage> {
        &self.image
    }


    /// Get the depth-stencil image view. (reference)
    #[inline]
    pub fn ref_image_view(&self) -> &Arc<ImageView<AttachmentImage>> {
        &self.view
    }
}


/// Get the depth-stencil format from the candidates.
/// Returns `None` if there is no format supported by the device.
/// 
/// Note: Modify this function to change which depth-stencil format you want to use...
/// 
#[inline]
fn get_depth_stencil_format(render_ctx: &RenderContext) -> Option<Format> {
    const CANDIDATE_FORMATS: [Format; 3] = [
        Format::D32_SFLOAT_S8_UINT,
        Format::D24_UNORM_S8_UINT,
        Format::D16_UNORM_S8_UINT,
    ];

    // checking that the candidate format is supported by the device.
    for format in CANDIDATE_FORMATS.into_iter() {
        if let Ok(properties) = render_ctx.get_format_properties(format) {
            if properties.optimal_tiling_features.contains(FormatFeatures::DEPTH_STENCIL_ATTACHMENT) {
                return Some(format)
            }
        }
    }
    return None;
}


/// Create a depth-stencil image and view.
/// 
/// # Runtime Errors 
/// - Returns a runtime error message if depth-stencil image creation fails.
/// - Returns a runtime error message if depth-stencil image view creation fails.
/// 
#[inline]
fn create_depth_stencil(
    width: u32, 
    height: u32, 
    format: Format, 
    allocator: &impl MemoryAllocator
) -> Result<(Arc<AttachmentImage>, Arc<ImageView<AttachmentImage>>), RuntimeError> {
    let image = AttachmentImage::with_usage(
        allocator, 
        [width, height], 
        format, 
        ImageUsage::DEPTH_STENCIL_ATTACHMENT
    ).map_err(|e| err!("Failed to create depth-stencil image: {}", e.to_string()))?;

    let view = ImageView::new(
        image.clone(),
        ImageViewCreateInfo {
            view_type: ImageViewType::Dim2d,
            format: Some(format),
            component_mapping: ComponentMapping::identity(),
            subresource_range: ImageSubresourceRange {
                aspects: ImageAspects::DEPTH | ImageAspects::STENCIL,
                mip_levels: (0..1),
                array_layers: (0..1)
            },
            ..Default::default()
        }
    ).map_err(|e| err!("Failed to create depth-stencil image view: {}", e.to_string()))?;

    Ok((image, view))
}