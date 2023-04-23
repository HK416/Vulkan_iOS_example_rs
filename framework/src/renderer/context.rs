use std::sync::Arc;

use vulkano::VulkanLibrary;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::format::{Format, FormatProperties};
use vulkano::memory::MemoryProperties;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::instance::{Instance, InstanceExtensions, InstanceCreateInfo};
use vulkano::device::{Device, Queue, Features, DeviceExtensions, QueueFlags, DeviceCreateInfo, QueueCreateInfo};
use vulkano::swapchain::{Surface, SurfaceInfo, SurfaceCapabilities, PresentMode, ColorSpace};

use crate::renderer::platform::*;
use crate::{err, error::RuntimeError};



#[derive(Debug)]
pub struct RenderContext {
    device: Arc<Device>,
    surface: Arc<Surface>,
    instance: Arc<Instance>,
    integrated_queue: Arc<Queue>, // <Graphics | Present | Compute>
    memory_allocator: StandardMemoryAllocator,
    descriptor_allocator: StandardDescriptorSetAllocator
}

impl RenderContext {
    /// Create a new `RenderContext`.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if the Vulkan library fails to load.
    /// - Returns a runtime error message if Vulkan instance creation fails.
    /// - Returns a runtime error message if no suitable device is found.
    /// - Returns a runtime error message if logical device creation fails.
    /// 
    pub fn new(handle: &AppHandle) -> Result<Arc<Self>, RuntimeError> {
        let instance = create_vulkan_instance()?;
        let surface = create_vulkan_surface(handle, &instance)?;
        let (device, integrated_queue) = create_vulkan_device_and_integrated_queue(
            &instance, 
            &surface
        )?;

        let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

        let descriptor_allocator = StandardDescriptorSetAllocator::new(device.clone());

        Ok(Arc::new(Self {
            device,
            surface,
            instance,
            integrated_queue,
            memory_allocator,
            descriptor_allocator,
        }))
    }


    /// Get the vulkan logical device. (reference)
    #[inline]
    pub fn ref_device(&self) -> &Arc<Device> {
        &self.device
    }


    /// Get the enabled features of the device. (reference)
    #[inline]
    pub fn ref_device_enabled_features(&self) -> &Features {
        self.device.enabled_features()
    }


    /// Get the enabled extensions of the device. (reference)
    #[inline]
    pub fn ref_device_enabled_extensions(&self) -> &DeviceExtensions {
        self.device.enabled_extensions()
    }


    /// Get the memory properties of the device. (reference)
    #[inline]   
    pub fn ref_device_memory_properties(&self) -> &MemoryProperties {
        self.device.physical_device().memory_properties()
    }


    /// Get the format properties of the device.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if getting format properties fails.
    /// 
    #[inline]
    pub fn get_format_properties(&self, format: Format) -> Result<FormatProperties, RuntimeError> {
        self.device
            .physical_device()
            .format_properties(format)
            .map_err(|e| err!("Failed to get format properties: {}", e.to_string()))
    }


    /// Get the vulkan surface. (reference)
    #[inline]
    pub fn ref_surface(&self) -> &Arc<Surface> {
        &self.surface
    }


    /// Get the surface capabilities of the device.
    /// 
    /// # Runtime Errors
    /// - Returns a runtime error message if getting surface capabilities fails.
    /// 
    #[inline]
    pub fn get_surface_capabilities(&self) -> Result<SurfaceCapabilities, RuntimeError> {
        self.device.physical_device()
            .surface_capabilities(&self.surface, SurfaceInfo::default())
            .map_err(|e| err!("Failed to get surface capabilities: {}", e.to_string()))
    }


    /// Get the surface present modes of the device.
    /// 
    /// # Runtime Errors 
    /// - Returns a runtime error message if getting surface present modes fails.
    /// 
    #[inline]
    pub fn get_surface_present_modes(&self) -> Result<impl Iterator<Item = PresentMode>, RuntimeError> {
        self.device.physical_device()
            .surface_present_modes(&self.surface)
            .map_err(|e| err!("Failed to get surface present modes: {}", e.to_string()))
    }


    /// Get the surface formats of the device.
    /// 
    /// # Runtime Errors 
    /// - Returns a runtime error message if getting suface formats fails.
    /// 
    #[inline]
    pub fn get_surface_formats(&self) -> Result<Vec<(Format, ColorSpace)>, RuntimeError>{
        self.device.physical_device()
            .surface_formats(&self.surface, SurfaceInfo::default())
            .map_err(|e| err!("Failed to get surface formats: {}", e.to_string()))
    }


    /// Get the vulkan queue. (Graphics, Present and Compute are integrated)
    #[inline]
    pub fn ref_integrated_queue(&self) -> &Arc<Queue> {
        &self.integrated_queue
    }


    /// Get the queue family index of the queue.
    #[inline]
    pub fn get_queue_fmaily_index(&self) -> u32 {
        self.integrated_queue.queue_family_index()
    }


    /// Get the standard memory allocator.
    #[inline]
    pub fn ref_memory_allocator(&self) -> &StandardMemoryAllocator {
        &self.memory_allocator
    }


    /// Get the standard descriptor allocator.
    #[inline]    
    pub fn ref_descriptor_allocator(&self) -> &StandardDescriptorSetAllocator {
        &self.descriptor_allocator
    }

    /// Get the standard command buffer allocator.
    #[inline]
    pub fn get_command_buffer_allocator(&self) -> StandardCommandBufferAllocator {
        StandardCommandBufferAllocator::new(
            self.device.clone(), 
            StandardCommandBufferAllocatorCreateInfo::default()
        )
    }
}



/// Load the Vulkan library.
/// 
/// # Runtime Errors
/// - Returns a runtime error message if the Vulkan library fails to load.
/// 
#[inline]
fn load_vulkan_library() -> Result<Arc<VulkanLibrary>, RuntimeError> {
    VulkanLibrary::new().map_err(|e| err!("Vk Library loading failed: {}", e.to_string()))
}


/// Get the enabled instance extension.
/// Unnecessary extensions are ignored when creating an instance.
/// 
/// Note: Modify this function to change which instance extension you want to use...
/// 
#[inline]
fn get_instance_extensions() -> InstanceExtensions {
    InstanceExtensions {
        khr_surface: true,
        khr_android_surface: true,
        khr_xcb_surface: true,
        khr_xlib_surface: true,
        khr_wayland_surface: true,
        khr_win32_surface: true,
        ext_metal_surface: true,
        khr_get_physical_device_properties2: true,
        khr_get_surface_capabilities2: true,
        ..Default::default()
    }
}


/// Create a vulkan instance.
/// 
/// # Runtime Errors
/// - Returns a runtime error message if the Vulkan library fails to load.
/// - Returns a runtime error message if Vulkan instance creation fails.
/// 
#[inline]
fn create_vulkan_instance() -> Result<Arc<Instance>, RuntimeError> {
    // load vulkan library.
    let library = load_vulkan_library()?;

    // get the enabled instance extensions.
    let enabled_extensions = library
        .supported_extensions()
        .intersection(&get_instance_extensions());

    // create vulkan instance.
    Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions,
            enumerate_portability: true, 
            ..Default::default()
        }
    ).map_err(|e| err!("Vulkan instance creation failed: {}", e.to_string()))
}


/// Get the enabled device extension.
/// If the device does not support extensions, it will not create the device.
/// 
/// Note: Modify this function to change which device extension you want to use...
/// 
#[inline]
fn get_device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    }
}


/// Get the enabled device features.
/// If the device does not support features, it will not create the device.
/// 
/// Note: Modify this function to change which device feature you want to use...
/// 
#[inline]
fn get_device_features() -> Features {
    Features {
        ..Default::default()
    }
}


/// Create a Vulkan logical device and integrated queue.
/// 
/// # Runtime Errors
/// - Returns a runtime error message if no suitable device is found.
/// - Returns a runtime error message if logical device creation fails.
/// 
#[inline]
fn create_vulkan_device_and_integrated_queue(
    instance: &Arc<Instance>, surface: &Arc<Surface>,
) -> Result<(Arc<Device>, Arc<Queue>), RuntimeError> {
    // get the enabled device extensions.
    let enabled_extensions = get_device_extensions();

    // get the enabled device features.
    let enabled_features = get_device_features();

    // get the suitable physical device and queue family index.
    let (physical_device, queue_family_index) = match instance
        .enumerate_physical_devices()
        .map_err(|e| err!("Physical device query failed: {}", e.to_string()))?
        .filter(|physical_device| {
            physical_device.supported_extensions().contains(&enabled_extensions)
            && physical_device.supported_features().contains(&enabled_features)
        })
        .filter_map(|physical_device| {
            physical_device.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(idx, properties)| {
                    properties.queue_flags.intersects(QueueFlags::GRAPHICS)
                    && physical_device.surface_support(idx as u32, surface).unwrap_or(false)
                })
                .map(|idx| (physical_device, idx as u32))
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
        })
    {
        Some(it) => it,
        None => return Err(err!("No suitable physical device found."))
    };

    // create Vulkan logical device and queues.
    let (device, mut queues) = Device::new(
        physical_device, 
        DeviceCreateInfo {
            enabled_extensions,
            enabled_features,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        }
    ).map_err(|e| err!("Vulkan device creation failed: {}", e.to_string()))?;

    Ok((device, queues.next().unwrap()))
}
