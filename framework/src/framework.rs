#![allow(unused_imports)]
use std::fmt::Debug;
use std::ffi::c_void;
use crate::timer::Timer;
use crate::renderer::Renderer;


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extent2D<T>
where T: Debug + Clone + Copy + PartialEq {
    pub width: T,
    pub height: T,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect<T> 
where T: Debug + Clone + Copy + PartialEq {
    pub top: T,
    pub left: T,
    pub bottom: T,
    pub right: T,
}


pub struct Framework {
    renderer: Renderer,
    timer: Timer,
}

impl Framework {
    #[cfg(target_os = "ios")]
    pub fn new_ios_ver(
        view: *mut c_void, 
        scale: f32,
        screen_size: Extent2D<u32>,
        viewer_area: Rect<i32>,
    ) -> Result<Self, String> {
        Ok(Self { 
            renderer: Renderer::new_ios_ver(view, scale, screen_size)?,
            timer: Timer::new(),
        })
    }

    pub fn frame_advanced(&mut self) -> Result<(), String> {
        self.timer.tick(None);
        self.renderer.draw(0.411765, 0.411765, 0.411765)?;
        println!("frame rate:{}", self.timer.get_frame_rate());
        Ok(())
    }

    pub fn paused(&mut self) -> Result<(), String> {
        self.timer.pause();
        println!("paused");
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        let elapsed_time = self.timer.resume();
        println!("resume (elapsed time: {}sec)", elapsed_time);
        Ok(())
    }
}