#![allow(unused_imports)]
mod renderer;
mod framework;

use std::ptr;
use std::ffi::{c_void, c_char};
use framework::{Framework, Extent2D, Rect};

static mut LAST_FRAMEWORK_ERR_MSG: Option<String> = None;


#[no_mangle]
#[cfg(target_os = "ios")]
pub extern "C" fn createFramework(
    view: *mut c_void,
    scale: f32,
    screen_width: u32,
    screen_height: u32,
    viewer_top: i32,
    viewer_left: i32,
    viewer_bottom: i32,
    viewer_right: i32,
) -> *mut c_void {
    assert!(!view.is_null(), "view cannot be a null pointer.");
    assert!(scale > 0.0, "scale must be greater than zero.");
    let screen_size = Extent2D { width: screen_width, height: screen_height };
    let viewer_area = Rect { top: viewer_top, left: viewer_left, bottom: viewer_bottom, right: viewer_right };
    return match Framework::new_ios_ver(view, scale, screen_size, viewer_area) {
        Ok(framework) => {
            Box::into_raw(Box::new(framework)) as *mut c_void
        },
        Err(msg) => {
            unsafe { LAST_FRAMEWORK_ERR_MSG = Some(msg) };
            ptr::null_mut()
        }
    };
}

#[no_mangle]
pub extern "C" fn destroyFramework(framework: *mut c_void) {
    assert!(!framework.is_null(), "framework cannot be a null pointer.");
    unsafe { Box::from_raw(framework as *mut Framework) };
}

#[no_mangle]
pub extern "C" fn updateFramework(framework: *mut c_void) -> *mut c_void {
    assert!(!framework.is_null(), "framework cannot be a null pointer.");
    let mut framework = unsafe { Box::from_raw(framework as *mut Framework) };
    return if let Err(msg) = framework.frame_advanced() {
        unsafe { LAST_FRAMEWORK_ERR_MSG = Some(msg) };
        ptr::null_mut()
    }
    else {
        Box::into_raw(framework) as *mut c_void
    };
}

#[no_mangle]
pub extern "C" fn pauseFramework(framework: *mut c_void) -> *mut c_void {
    assert!(!framework.is_null(), "framework cannot be a null pointer.");
    let mut framework = unsafe { Box::from_raw(framework as *mut Framework) };
    return if let Err(msg) = framework.paused() {
        unsafe { LAST_FRAMEWORK_ERR_MSG = Some(msg) };
        ptr::null_mut()
    }
    else {
        Box::into_raw(framework) as *mut c_void
    };
}

#[no_mangle]
pub extern "C" fn resumeFramework(framework: *mut c_void) -> *mut c_void {
    assert!(!framework.is_null(), "framework cannot be a null pointer.");
    let mut framework = unsafe { Box::from_raw(framework as *mut Framework) };
    return if let Err(msg) = framework.resume() {
        unsafe { LAST_FRAMEWORK_ERR_MSG = Some(msg) };
        ptr::null_mut()
    }
    else {
        Box::into_raw(framework) as *mut c_void
    };
}

#[no_mangle]
pub extern "C" fn getLastFrameworkErrMsg(buf: *mut c_char, buf_size: u32) -> bool {
    assert!(!buf.is_null(), "buffer cannot be a null pointer.");
    assert!(buf_size > 0, "buffer size cannot be zero.");
    return match unsafe { &LAST_FRAMEWORK_ERR_MSG } {
        Some(msg) => {
            unsafe { buf.copy_from(msg.as_ptr() as *const i8, buf_size as usize) };
            true
        },
        None => false
    };
}
