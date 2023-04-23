#![allow(unused_imports)]
use std::ffi::c_void;
use std::path::PathBuf;

use crate::timer::*;
use crate::renderer::*;
use crate::world::scene::SceneManager;
use crate::{err, error::RuntimeError};

use crate::app::*;


#[derive(Debug)]
pub struct Framework {
    timer: Timer,
    renderer: Renderer,
    scene_manager: SceneManager,
}

impl Framework {
    pub fn new(
        handle: AppHandle, 
        assets_dir: PathBuf,
        scale_factor: f32,
        screen_size: (u32, u32),
        viewer_area: (i32, i32, i32, i32),
    ) -> Result<Self, RuntimeError> {
        let timer = Timer::new();
        let renderer = Renderer::new(handle, &assets_dir, scale_factor, screen_size, viewer_area)?;
        let scene_manager = SceneManager::new(
            [("Main".to_string(), MainScene::new() as _)],
            "Main".to_string(),
            &renderer
        )?;

        Ok(Self {
            timer,
            renderer,
            scene_manager,
        })
    }

    pub fn frame_advanced(&mut self) -> Result<(), RuntimeError> {
        self.timer.tick(Some(60));
        self.scene_manager.frame_advanced(&mut self.timer, &mut self.renderer)?;
        
        #[cfg(feature = "monitor")]
        println!("<monitor> frame_rate={}", self.timer.get_frame_rate());
        
        Ok(())
    }

    pub fn paused(&mut self) -> Result<(), RuntimeError> {
        self.timer.pause();
        self.scene_manager.pause(&self.timer, &self.renderer)?;

        #[cfg(feature = "monitor")]
        println!("<monitor> framework paused. (total_time={}sec)", self.timer.get_elapsed_time_in_sec());

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), RuntimeError> {
        let _total_time = self.timer.get_total_time_in_sec();
        let _elapsed_time = self.timer.resume();
        self.scene_manager.resume(&self.timer, &self.renderer)?;
        
        #[cfg(feature = "monitor")]
        println!("<monitor> framework resume. (total_time={}sec, duration={}sec)", _total_time, _elapsed_time);

        Ok(())
    }
}