#![allow(unused_imports)]
mod math;
mod mesh;
mod timer;
mod error;
mod renderer;
mod framework;

use std::ptr;
use std::ffi::{c_void, c_char};

use error::RuntimeError;
use renderer::AppHandle;
use framework::Framework;

static mut LAST_FRAMEWORK_ERR_MSG: Option<RuntimeError> = None;

#[no_mangle]
#[cfg(target_os = "ios")]
pub extern "C" fn createFramework(
    ui_view: *mut c_void,
    screen_width: u32,
    screen_height: u32,
    viewer_top: i32,
    viewer_left: i32,
    viewer_bottom: i32,
    viewer_right: i32,
) -> *mut c_void {
    assert!(!ui_view.is_null(), "view cannot be a null pointer.");
    let handle = AppHandle::IOS { ui_view: unsafe { std::mem::transmute(ui_view) } };
    let screen_size = (screen_width, screen_height);
    let viewer_area = (viewer_top, viewer_left, viewer_bottom, viewer_right);
    return match Framework::new(handle, Some(screen_size), Some(viewer_area)) {
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
            unsafe { buf.copy_from(msg.what().as_ptr() as *const i8, buf_size as usize) };
            true
        },
        None => false
    };
}

#[no_mangle]
pub extern "C" fn getLastFrameworkErrMsgDbg(buf: *mut c_char, buf_size: u32) -> bool {
    assert!(!buf.is_null(), "buffer cannot be a null pointer.");
    assert!(buf_size > 0, "buffer size cannot be zero.");
    return match unsafe { &LAST_FRAMEWORK_ERR_MSG } {
        Some(msg) => {
            unsafe { buf.copy_from(msg.debug_info().as_ptr() as *const i8, buf_size as usize) };
            true
        },
        None => false
    };
}