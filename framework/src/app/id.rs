use rand::Rng;
use rand::distributions::{Distribution, Standard};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshID {
    Triangle = 0,
    Quad = 1,
    Cube = 2,
}

impl Distribution<MeshID> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MeshID {
        match rng.gen_range(0..3) {
            0 => MeshID::Triangle,
            1 => MeshID::Quad,
            _ => MeshID::Cube,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemID {
    Rotation = 0,
}

impl Distribution<SystemID> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SystemID {
        match rng.gen_range(0..1) {
            _ => SystemID::Rotation,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderID {
    Default = 0,
}

impl Distribution<ShaderID> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ShaderID {
        match rng.gen_range(0..1) {
            _ => ShaderID::Default,
        }
    }
}
