#![allow(unused_imports)]
use std::fmt::Debug;
use std::ffi::c_void;
use crate::timer::Timer;
use crate::error::RuntimeError;
use crate::renderer::{rgb, Renderer, AppHandle};


pub struct Framework {
    renderer: Renderer,
    timer: Timer,
    viewer_area: Option<(i32, i32, i32, i32)>,
}

impl Framework {
    pub fn new(
        handle: AppHandle, 
        screen_size: Option<(u32, u32)>,
        viewer_area: Option<(i32, i32, i32, i32)>
    ) -> Result<Self, RuntimeError> {
        let renderer = Renderer::new(handle, screen_size)?;
        let timer = Timer::new();
        Ok(Self {
            renderer,
            timer,
            viewer_area, 
        })
    }

    pub fn frame_advanced(&mut self) -> Result<(), RuntimeError> {
        self.timer.tick(None);

        let (red, green, blue) = rgb(128, 128, 128);
        if let Some(guard) = self.renderer.prepare_render(red, green, blue, 1.0)? {
            self.renderer.submit_and_present(guard)?;
        }

        println!("frame rate:{}", self.timer.get_frame_rate());
        Ok(())
    }

    pub fn paused(&mut self) -> Result<(), RuntimeError> {
        self.timer.pause();
        println!("paused");
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), RuntimeError> {
        let elapsed_time = self.timer.resume();
        println!("resume (elapsed time: {}sec)", elapsed_time);
        Ok(())
    }
}