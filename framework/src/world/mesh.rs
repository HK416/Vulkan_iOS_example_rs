pub use crate::math::*;
pub use crate::renderer::*;

pub trait ModelMesh {
    fn prepare_drawing(&self, builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>) -> Result<(), RuntimeError>;

    fn draw(&self, builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>) -> Result<(), RuntimeError>;
}
