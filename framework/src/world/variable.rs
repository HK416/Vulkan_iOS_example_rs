use std::fmt;
use std::mem;
use std::ptr;
use std::sync::Arc;

use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::buffer::{Subbuffer, BufferContents, Buffer, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{MemoryAllocator, AllocationCreateInfo, MemoryUsage};

use crate::{err, error::RuntimeError};



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaderVariableAccess {
    Buffer(Subbuffer<[u8]>),
}



pub trait ShaderVariableAbstract : fmt::Debug + Send + Sync {
    fn write_descriptor(&self, binding: u32) -> WriteDescriptorSet;
    fn access(&self) -> ShaderVariableAccess;
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniformBuffer<T> 
where T: fmt::Debug + BufferContents {
    buffer: Subbuffer<T>
}


impl<T> UniformBuffer<T> 
where T: fmt::Debug + BufferContents {
    #[inline]
    pub fn uninit(allocator: &impl MemoryAllocator) -> Result<Arc<Self>, RuntimeError> {
        Ok(Arc::new(Self {
            buffer: Buffer::new_unsized(
                allocator,
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    usage: MemoryUsage::Upload,
                    ..Default::default()
                },
                mem::size_of::<T>() as u64,
            ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?
        }))
    }

    #[inline]
    pub fn from_data(
        data: T,
        allocator: &impl MemoryAllocator,
    ) -> Result<Arc<Self>, RuntimeError> {
        Ok(Arc::new(Self {
            buffer: Buffer::from_data(
                allocator,
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    usage: MemoryUsage::Upload,
                    ..Default::default()
                },
                data
            ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?
        }))
    }

    #[inline]
    pub fn write_data(&self, data: T) {
        if let Some(ptr) = self.buffer.mapped_ptr() {
            unsafe { 
                std::ptr::write(
                    ptr.cast().as_ptr(), 
                    data
                );
            };
        }
    }
}


impl<T> ShaderVariableAbstract for UniformBuffer<T>
where T: fmt::Debug + BufferContents {
    fn write_descriptor(&self, binding: u32) -> WriteDescriptorSet {
        WriteDescriptorSet::buffer(binding, self.buffer.clone())
    }

    #[inline]
    fn access(&self) -> ShaderVariableAccess {
        ShaderVariableAccess::Buffer(self.buffer.as_bytes().clone())
    }
}