#![allow(unused_imports)]
use std::fmt::Debug;
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use vulkano::command_buffer::{RenderingInfo, PrimaryCommandBufferAbstract};
use vulkano::sync::GpuFuture;

use crate::timer::Timer;
use crate::world::scene::SceneManager;
use crate::{err, error::RuntimeError};
use crate::renderer::{Renderer, AppHandle};

use crate::app::*;


pub struct Framework {
    timer: Timer,
    renderer: Renderer,
    scene_manager: SceneManager<SceneID>,
}

impl Framework {
    pub fn new(
        handle: AppHandle, 
        assets_dir: PathBuf,
        scale_factor: f32,
        screen_size: [u32; 2],
        viewer_area: [i32; 4],
    ) -> Result<Self, RuntimeError> {
        let timer = Timer::new();
        let renderer = Renderer::new(handle, &assets_dir, scale_factor, screen_size, viewer_area)?;

        let mut builder = renderer.primary_command_buffer(None)?;

        let scene_manager = SceneManager::new(vec![
            (SceneID::Main, MainScene::new(&renderer, &mut builder)?),
        ]);

        let mut future = renderer.queue_submit(builder)?;
        future.cleanup_finished();

        Ok(Self {
            timer,
            renderer,
            scene_manager,
        })
    }

    pub fn frame_advanced(&mut self) -> Result<(), RuntimeError> {
        self.timer.tick(Some(60));
        self.scene_manager.frame_advanced(&mut self.timer, &mut self.renderer)?;
        println!("frame rate:{}", self.timer.get_frame_rate());
        Ok(())
    }

    pub fn paused(&mut self) -> Result<(), RuntimeError> {
        self.timer.pause();
        self.scene_manager.pause(&self.timer, &self.renderer)?;
        println!("paused");
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), RuntimeError> {
        let elapsed_time = self.timer.resume();
        self.scene_manager.resume(&self.timer, &self.renderer)?;
        println!("resume (elapsed time: {}sec)", elapsed_time);
        Ok(())
    }
}