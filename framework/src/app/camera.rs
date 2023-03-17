use crate::world::model::*;

#[derive(Debug, Clone, PartialEq)]
pub struct PerspectiveCamera {
    scissor: Scissor,
    viewport: Viewport,
    transform: Mat4x4,
}

impl PerspectiveCamera {
    #[inline]
    pub fn new(
        scissor: Scissor,
        viewport: Viewport,
        position: Vec3,
        quaternion: Quat,
    ) -> Arc<Mutex<Self>> {
        let mut transform = quaternion.into_matrix4x4();
        transform.r4c1 = position.x;
        transform.r4c2 = position.y;
        transform.r4c3 = position.z;

        Arc::new(Mutex::new(Self { scissor, viewport, transform }))
    }
}

impl Model for PerspectiveCamera {
    #[inline]
    fn ref_world_matrix(&self) -> &Mat4x4 {
        &self.transform
    }

    #[inline]
    fn mut_world_matrix(&mut self) -> &mut Mat4x4 {
        &mut self.transform
    }
}

impl CameraModel for PerspectiveCamera {
    #[inline]
    fn ref_viewport(&self) -> &Viewport {
        &self.viewport
    }

    #[inline]
    fn mut_viewport(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    #[inline]
    fn ref_scissor(&self) -> &Scissor {
        &self.scissor
    }

    #[inline]
    fn mut_scissor(&mut self) -> &mut Scissor {
        &mut self.scissor
    }

    #[inline]
    fn get_projection_matrix(&self) -> Mat4x4 {
        // orthographic_rh_zo(-2.0, 2.0, -2.0, 2.0, 0.01, 1000.0)
        let aspect = self.viewport.dimensions[0] / self.viewport.dimensions[1];
        perspective_rh_zo(60_f32.to_radians(), aspect, 0.01, 1000.0)
    }
}
