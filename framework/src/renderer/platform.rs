use std::sync::Arc;
use std::ffi::c_void;

use vulkano::instance::Instance;
use vulkano::swapchain::Surface;

use crate::{err, error::RuntimeError};

#[cfg(any(target_os = "ios", target_os = "macos"))]
use self::apple::*;



/// Application native handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppHandle {
    IOS { ui_view: *mut Object },
    MacOS { ns_view: *mut Object },
}

impl AppHandle {
    /// Creates an iOS handle with the given UIView pointer.
    /// A given UIView must implement CAMetalLayer.
    /// 
    /// # Unsafety 
    /// The given pointer must be a valid UIView pointer.
    /// Libraries are not checked for correctness.
    /// 
    #[inline]
    #[cfg(target_os = "ios")]
    pub unsafe fn from_ios(ui_view: *mut c_void) -> Self {
        Self::IOS { ui_view: std::mem::transmute(ui_view) }
    }

    /// Creates an macOS handle with the given NSView pointer.
    /// A given NSView must implement CAMetalLayer.
    /// 
    /// # Unsafety 
    /// The given pointer must be valid NSView pointer.
    /// Libraries are not checked for correctness.
    /// 
    #[inline]
    #[cfg(target_os = "macos")]
    pub unsafe fn from_macos(ns_view: *mut c_void) -> Self {
        Self::MacOS { ns_view: std::mem::transmute(ns_view) }
    }
}

unsafe impl Send for AppHandle { }
unsafe impl Sync for AppHandle { }


/// Creates a vulkan surface with the given application handle.
/// 
/// # Runtime Errors
/// - If creation fails, a runtime error message is returned.
/// 
/// # Panics
/// - (macOS or iOS) Abort program execution if the pointer is not valid.
/// 
#[inline]
pub fn create_vulkan_surface(
    handle: &AppHandle,
    instance: &Arc<Instance>
) -> Result<Arc<Surface>, RuntimeError> {
    match handle {
        #[cfg(target_os = "ios")]
        &AppHandle::IOS { ui_view } => {
            unsafe { create_vulkan_surface_ios(ui_view, instance) }
        },
        #[cfg(target_os = "macos")]
        &AppHandle::MacOS { ns_view } => {
            unsafe { create_vulkan_surface_macos(ns_view, instance) }
        },
        _ => Err(err!("No supported platform."))
    }
}


/// A function that creates a vulkan surface for iOS.
/// 
/// # Runtime Errors
/// - If creation fails, a runtime error message is returned.
/// 
/// # Panics
/// - Abort program execution if the pointer is not valid.
/// 
#[inline]
#[cfg(target_os = "ios")]
unsafe fn create_vulkan_surface_ios(
    ui_view: *mut Object,
    instance: &Arc<Instance>
) -> Result<Arc<Surface>, RuntimeError> {
    let layer: *mut Object = msg_send![ui_view, layer];
    create_vulkan_surface_metal(layer, instance)
}


/// A function that creates a vulkan surface for macOS.
/// 
/// # Runtime Errors
/// - If creation fails, a runtime error message is returned.
/// 
/// # Panics
/// - Abort program execution if the pointer is not valid.
/// 
#[inline]
#[cfg(target_os = "macos")]
unsafe fn create_vulkan_surface_macos(
    ns_view: *mut Object,
    instance: &Arc<Instance>
) -> Result<Arc<Surface>, RuntimeError> {
    let layer: *mut Object = msg_send![ns_view, layer];
    create_vulkan_surface_metal(layer, instance)
}


/// A function that creates a vulkan surface for apple metal.
/// 
/// # Runtime Errors
/// - If creation fails, a runtime error message is returned.
/// 
#[inline]
#[cfg(any(target_os = "macos", target_os = "ios"))]
unsafe fn create_vulkan_surface_metal(
    layer: *mut Object,
    instance: &Arc<Instance>
) -> Result<Arc<Surface>, RuntimeError> {
    Surface::from_metal(
        instance.clone(), 
        layer, 
        None
    ).map_err(|e| err!("Vk Create Error: {}", e.to_string()))
}



#[cfg(any(target_os = "ios", target_os = "macos"))]
mod apple {
    use std::ffi;
    use std::fmt;
    pub use objc::{msg_send, class, sel, sel_impl, runtime::{ Object, YES, NO }};

    #[cfg(target_pointer_width = "32")]
    pub type CGFloat = ffi::c_float;
    #[cfg(target_pointer_width = "64")]
    pub type CGFloat = ffi::c_double;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct CGPoint {
        pub x: CGFloat,
        pub y: CGFloat,
    }

    impl fmt::Debug for CGPoint {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("CGPoint")
                .field("x", &self.x)
                .field("y", &self.y)
                .finish()
        }
    }
    
    impl fmt::Display for CGPoint {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "({}, {})", &self.x, &self.y)
        }
    }
    
    unsafe impl objc::Encode for CGPoint {
        fn encode() -> objc::Encoding {
            let encoding = format!("{{CGPoint={}{}}}", CGFloat::encode().as_str(), CGFloat::encode().as_str());
            unsafe { objc::Encoding::from_str(&encoding) }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct CGSize {
        pub width: CGFloat,
        pub height: CGFloat,
    }

    impl fmt::Debug for CGSize {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("CGSize")
                .field("width", &self.width)
                .field("height", &self.height)
                .finish()
        }
    }
    
    impl fmt::Display for CGSize {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "({}, {})", &self.width, &self.height)
        }
    }
    
    unsafe impl objc::Encode for CGSize {
        fn encode() -> objc::Encoding {
            let encoding = format!("{{CGSize={}{}}}", CGFloat::encode().as_str(), CGFloat::encode().as_str());
            unsafe { objc::Encoding::from_str(&encoding) }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct CGRect {
        pub origin: CGPoint,
        pub size: CGSize,
    }

    impl fmt::Debug for CGRect {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("CGRect")
                .field("origin", &self.origin)
                .field("size", &self.size)
                .finish()
        }
    }
    
    impl fmt::Display for CGRect {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "({}, {})", &self.origin, &self.size)
        }
    }
    
    unsafe impl objc::Encode for CGRect {
        fn encode() -> objc::Encoding {
            let encoding = format!("{{CGRect={}{}}}", CGPoint::encode().as_str(), CGSize::encode().as_str());
            unsafe { objc::Encoding::from_str(&encoding) }
        }
    }
}
