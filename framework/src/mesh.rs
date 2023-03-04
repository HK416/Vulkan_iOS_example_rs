use std::sync::Arc;

use vulkano::buffer::DeviceLocalBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;

pub trait Mesh {
    fn prepare_render<T>(&self, builder: &mut AutoCommandBufferBuilder<T>) {
    }
}

