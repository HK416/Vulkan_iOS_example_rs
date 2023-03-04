use std::sync::Arc;
use std::ffi::c_void;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use crate::{err, error::RuntimeError};

#[cfg(any(target_os = "ios", target_os = "macos"))]
use apple::*;

#[cfg(target_os = "ios")]
use vulkano::swapchain::IOSMetalLayer;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppHandle {
    IOS { ui_view: *mut Object },
    MacOS { ns_view: *mut Object },
}

impl AppHandle {
    #[inline]
    pub fn from_ios(ui_view: *mut c_void) -> Self {
        Self::IOS { ui_view: unsafe { std::mem::transmute(ui_view) }}
    }

    #[inline]
    pub fn from_macos(ns_view: *mut c_void) -> Self {
        Self::MacOS { ns_view: unsafe { std::mem::transmute(ns_view) }}
    }
}

#[inline]
#[cfg(target_os = "ios")]
unsafe fn create_vulkan_surface_ios(
    ui_view: *mut Object,
    instance: &Arc<Instance>
) -> Result<Arc<Surface>, RuntimeError> {
    let main_layer: *mut Object = msg_send![ui_view, layer];
    return Surface::from_ios(
        instance.clone(),
        IOSMetalLayer::new(ui_view, main_layer),
        None
    ).map_err(|e| err!("Vk Create Error:{}", e.to_string()));
}

#[inline]
#[cfg(target_os = "macos")]
unsafe fn create_vulkan_surface_macos(
    ns_view: *mut Object,
    instance: &Arc<Instance>
) -> Result<Arc<Surface>, RuntimeError> {
    return Surface::from_mac_os(
        instance.clone(), 
        ns_view, 
        None
    ).map_err(|e| err!("Vk Create Error:{}", e.to_string()));
}

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

#[inline]
#[cfg(target_os = "ios")]
unsafe fn get_screen_size_ios(ui_view: *mut Object) 
-> Result<(u32, u32), RuntimeError> {
    let bounds: CGRect = msg_send![ui_view, bounds];
    Ok((bounds.size.width as u32, bounds.size.height as u32))
}

#[inline]
#[cfg(target_os = "macos")]
unsafe fn get_screen_size_macos(ns_view: *mut Object) 
-> Result<(u32, u32), RuntimeError> {
    let bounds: CGRect = msg_send![ns_view, bounds];
    Ok((bounds.size.width as u32, bounds.size.height as u32))
}

#[inline]
pub fn get_screen_size(handle: &AppHandle) 
-> Result<(u32, u32), RuntimeError> {
    match handle {
        #[cfg(target_os = "ios")]
        &AppHandle::IOS { ui_view } => {
            unsafe { get_screen_size_ios(ui_view) }
        },
        #[cfg(target_os = "macos")]
        &AppHandle::MacOS { ns_view } => {
            unsafe { get_screen_size_macos(ns_view) }
        },
        _ => Err(err!("No supported platform."))
    }
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
