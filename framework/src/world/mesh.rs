use std::fmt;
use std::mem;
use std::sync::Arc;

use bytemuck::offset_of;
use vulkano::format::Format;
use vulkano::buffer::{Buffer, BufferUsage, BufferContents, BufferCreateInfo, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CopyBufferInfo};
use vulkano::command_buffer::allocator::CommandBufferAllocator;
use vulkano::pipeline::graphics::vertex_input::{VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, VertexInputState};

use crate::math::*;
use crate::renderer::RenderContext;
use crate::{err, error::RuntimeError};



/// Index buffer data type.
/// Either 16-bit unsigned integer type or 
/// 32-bit unsigned integer type can be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexBuffer {
    U16(Subbuffer<[u16]>),
    U32(Subbuffer<[u32]>)
}

impl IndexBuffer {
    /// Create an index buffer from 16-bit unsigned integer index data.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while creating the index buffer.
    /// 
    #[inline]
    pub fn from_iter_u16<L, A, I>(
        iter: I,
        allocator: &impl MemoryAllocator,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Self, RuntimeError> 
    where 
        A: CommandBufferAllocator, 
        I: IntoIterator<Item = u16>, 
        I::IntoIter: ExactSizeIterator, 
    {
        let staging_buffer = Buffer::from_iter(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            }, 
            iter
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        let buffer = Buffer::new_unsized(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            }, 
            staging_buffer.size()
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        command_buffer_builder.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer,
            buffer.clone()
        )).map_err(|e| err!("Buffer copy failed: {}", e.to_string()))?;
        
        Ok(Self::U16(buffer))
    }

    /// Create an index buffer from 32-bit unsigned integer index data.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while creating the index buffer.
    /// 
    #[inline]
    pub fn from_iter_u32<L, A, I>(
        iter: I,
        allocator: &impl MemoryAllocator,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Self, RuntimeError> 
    where 
        A: CommandBufferAllocator, 
        I: IntoIterator<Item = u32>, 
        I::IntoIter: ExactSizeIterator 
    {
        let staging_buffer = Buffer::from_iter(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            }, 
            iter
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        let buffer = Buffer::new_unsized(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            }, 
            staging_buffer.size()
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        command_buffer_builder.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer,
            buffer.clone()
        )).map_err(|e| err!("Buffer copy failed: {}", e.to_string()))?;
        
        Ok(Self::U32(buffer))
    }
}



/// Interface of vertex buffer.
pub trait VertexBufferAbstract : fmt::Debug + Send + Sync {
    /// Stride of the buffer.
    fn stride(&self) -> u32;

    /// Format of the buffer.
    fn format(&self) -> &[(Format, u32)];

    /// Input rate of the buffer.
    fn input_rate(&self) -> VertexInputRate;

    /// buffer access
    fn buffer_access(&self) -> Subbuffer<[u8]>;
}



/// A vertex buffer that creates a buffer in device local memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuVertexBuffer<T> 
where T: fmt::Debug, [T]: BufferContents {
    stride: u32,
    format: Vec<(Format, u32)>,
    input_rate: VertexInputRate,
    buffer: Subbuffer<[T]>
}

impl GpuVertexBuffer<Vec2> {
    /// Create an vertex buffer from `Vec2` vertex data.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while creating the vertex buffer.
    /// 
    #[inline]
    pub fn from_iter_vec2<L, A, I>(
        iter: I, 
        input_rate: VertexInputRate,
        allocator: &impl MemoryAllocator,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<Self>, RuntimeError> 
    where 
        A: CommandBufferAllocator, 
        I: IntoIterator<Item = Vec2>, 
        I::IntoIter: ExactSizeIterator 
    {
        let staging_buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            iter
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        let buffer = Buffer::new_unsized(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            }, 
            staging_buffer.size()
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        command_buffer_builder.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer, 
            buffer.clone()
        )).map_err(|e| err!("Buffer copy failed: {}", e.to_string()))?;

        Ok(Arc::new(Self {
            stride: mem::size_of::<Vec2>() as u32,
            format: vec![(Format::R32G32_SFLOAT, 0)],
            input_rate,
            buffer,
        }))
    }
}

impl GpuVertexBuffer<Vec3> {
    /// Create an vertex buffer from `Vec3` vertex data.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while creating the vertex buffer.
    /// 
    #[inline]
    pub fn from_iter_vec3<L, A, I>(
        iter: I,
        input_rate: VertexInputRate,
        allocator: &impl MemoryAllocator,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<Self>, RuntimeError> 
    where 
        A: CommandBufferAllocator, 
        I: IntoIterator<Item = Vec3>, 
        I::IntoIter: ExactSizeIterator 
    {
        let staging_buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            iter
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        let buffer = Buffer::new_unsized(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            }, 
            staging_buffer.size()
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        command_buffer_builder.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer, 
            buffer.clone()
        )).map_err(|e| err!("Buffer copy failed: {}", e.to_string()))?;

        Ok(Arc::new(Self {
            stride: mem::size_of::<Vec3>() as u32,
            format: vec![(Format::R32G32B32_SFLOAT, 0)],
            input_rate,
            buffer,
        }))
    }
}    

impl GpuVertexBuffer<Vec4> {
    /// Create an vertex buffer from `Vec4` vertex data.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while creating the vertex buffer.
    /// 
    #[inline]
    pub fn from_iter_vec4<L, A, I>(
        iter: I,
        input_rate: VertexInputRate,
        allocator: &impl MemoryAllocator,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<Self>, RuntimeError>
    where 
        A: CommandBufferAllocator, 
        I: IntoIterator<Item = Vec4>, 
        I::IntoIter: ExactSizeIterator 
    {
        let staging_buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            iter
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        let buffer = Buffer::new_unsized(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            }, 
            staging_buffer.size()
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        command_buffer_builder.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer, 
            buffer.clone()
        )).map_err(|e| err!("Buffer copy failed: {}", e.to_string()))?;

        Ok(Arc::new(Self {
            stride: mem::size_of::<Vec4>() as u32,
            format: vec![(Format::R32G32B32A32_SFLOAT, 0)],
            input_rate,
            buffer,
        }))
    }
}

impl GpuVertexBuffer<Mat3x3> {
    /// Create an vertex buffer from `Mat3x3` vertex data.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while creating the vertex buffer.
    /// 
    #[inline]
    pub fn from_iter_mat3<L, A, I>(
        iter: I,
        input_rate: VertexInputRate,
        allocator: &impl MemoryAllocator,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<Self>, RuntimeError> 
    where 
        A: CommandBufferAllocator, 
        I: IntoIterator<Item = Mat3x3>, 
        I::IntoIter: ExactSizeIterator 
    {
        let staging_buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            iter
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        let buffer = Buffer::new_unsized(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            }, 
            staging_buffer.size()
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        command_buffer_builder.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer, 
            buffer.clone()
        )).map_err(|e| err!("Buffer copy failed: {}", e.to_string()))?;

        Ok(Arc::new(Self {
            stride: mem::size_of::<Mat3x3>() as u32,
            format: vec![
                (Format::R32G32B32_SFLOAT, offset_of!(Mat3x3, r1c1) as u32),
                (Format::R32G32B32_SFLOAT, offset_of!(Mat3x3, r2c1) as u32),
                (Format::R32G32B32_SFLOAT, offset_of!(Mat3x3, r3c1) as u32),
            ],
            input_rate,
            buffer, 
        }))
    }
}

impl GpuVertexBuffer<Mat4x4> {
    /// Create an vertex buffer from `Mat4x4` vertex data.
    /// 
    /// # Runtime Error
    /// Return the `RuntimeError` if an error occurs while creating the vertex buffer.
    /// 
    #[inline]
    pub fn from_iter_mat4<L, A, I>(
        iter: I,
        input_rate: VertexInputRate,
        allocator: &impl MemoryAllocator,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<Self>, RuntimeError> 
    where 
        A: CommandBufferAllocator, 
        I: IntoIterator<Item = Mat4x4>, 
        I::IntoIter: ExactSizeIterator 
    {
        let staging_buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            iter
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        let buffer = Buffer::new_unsized(
            allocator, 
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            }, 
            staging_buffer.size()
        ).map_err(|e| err!("Buffer creation failed: {}", e.to_string()))?;

        command_buffer_builder.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer, 
            buffer.clone()
        )).map_err(|e| err!("Buffer copy failed: {}", e.to_string()))?;

        Ok(Arc::new(Self {
            stride: mem::size_of::<Mat4x4>() as u32,
            format: vec![
                (Format::R32G32B32A32_SFLOAT, offset_of!(Mat4x4, r1c1) as u32),
                (Format::R32G32B32A32_SFLOAT, offset_of!(Mat4x4, r2c1) as u32),
                (Format::R32G32B32A32_SFLOAT, offset_of!(Mat4x4, r3c1) as u32),
                (Format::R32G32B32A32_SFLOAT, offset_of!(Mat4x4, r4c1) as u32),
            ],
            input_rate,
            buffer, 
        }))
    }
}


impl<T> VertexBufferAbstract for GpuVertexBuffer<T> 
where T: fmt::Debug, [T]: BufferContents {
    #[inline]
    fn stride(&self) -> u32 {
        self.stride
    }

    #[inline]
    fn format(&self) -> &[(Format, u32)] {
        &self.format
    }

    #[inline]
    fn input_rate(&self) -> VertexInputRate {
        self.input_rate
    }

    fn buffer_access(&self) -> Subbuffer<[u8]> {
        self.buffer.as_bytes().clone()
    }
}



/// `Mesh` object used in `Model`.
#[derive(Debug, Clone)]
pub struct Mesh {
    index_count: u32,
    vertex_count: u32,
    index_buffer: Option<IndexBuffer>,
    vertex_buffers: Vec<Arc<dyn VertexBufferAbstract>>,
    vertex_input_state: VertexInputState,
}

impl Mesh {
    /// Creates a new mesh from vertex buffers.
    pub fn new<Iter>(
        vertex_count: u32,
        vertex_buffers: Iter
    ) -> Arc<Self>
    where Iter: IntoIterator<Item = Arc<dyn VertexBufferAbstract>>, Iter::IntoIter: ExactSizeIterator {
        let vertex_buffers: Vec<_> = vertex_buffers.into_iter().collect();
        let (bindings, attributes): (Vec<_>, Vec<Vec<_>>) = vertex_buffers
            .iter()
            .enumerate()
            .map(|(i, buffer)| {(
                VertexInputBindingDescription {
                    input_rate: buffer.input_rate(),
                    stride: buffer.stride()
                },
                buffer.format().iter()
                    .map(|&(format, offset)| {
                        VertexInputAttributeDescription {
                            binding: i as u32,
                            format,
                            offset
                        }
                    })
                    .collect()
            )})
            .unzip();
        
        let vertex_input_state = VertexInputState::new()
            .bindings(bindings.into_iter().enumerate().map(|(i, description)| {
                (i as u32, description)
            }))
            .attributes(attributes.into_iter().flatten().enumerate().map(|(i, description)| {
                (i as u32, description)
            }));

        Arc::new(Self {
            index_count: 0,
            index_buffer: None,
            vertex_count,
            vertex_buffers,
            vertex_input_state,
        })
    }

    /// Creates a new mesh from index buffer and vertex buffers.
    pub fn new_with_index<Iter>(
        index_count: u32,
        index_buffer: IndexBuffer,
        vertex_count: u32,
        vertex_buffers: Iter
    ) -> Arc<Self>
    where Iter: IntoIterator<Item = Arc<dyn VertexBufferAbstract>>, Iter::IntoIter: ExactSizeIterator {
        let vertex_buffers: Vec<_> = vertex_buffers.into_iter().collect();
        let (bindings, attributes): (Vec<_>, Vec<Vec<_>>) = vertex_buffers
            .iter()
            .enumerate()
            .map(|(i, buffer)| {(
                VertexInputBindingDescription {
                    input_rate: buffer.input_rate(),
                    stride: buffer.stride()
                },
                buffer.format().iter()
                    .map(|&(format, offset)| {
                        VertexInputAttributeDescription {
                            binding: i as u32,
                            format,
                            offset
                        }
                    })
                    .collect()
            )})
            .unzip();
        
        let vertex_input_state = VertexInputState::new()
            .bindings(bindings.into_iter().enumerate().map(|(i, description)| {
                (i as u32, description)
            }))
            .attributes(attributes.into_iter().flatten().enumerate().map(|(i, description)| {
                (i as u32, description)
            }));

        Arc::new(
            Self {
                index_count,
                index_buffer: Some(index_buffer),
                vertex_count,
                vertex_buffers,
                vertex_input_state,
            }
        )
    }

    /// Borrow the `VertexInputState`.
    #[inline]
    pub fn get_vertex_input_state(&self) -> &VertexInputState {
        &self.vertex_input_state
    }

    /// Bind the mesh's buffer to the command buffer.
    /// 
    /// # Unsafety
    /// You must to bind the mesh's buffer to the command buffer and then call the draw command.
    /// Otherwise, the mesh may not be drawn normally.
    /// 
    #[inline]
    pub unsafe fn bind_buffers<L, A: CommandBufferAllocator>(
        &self, command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) {
        // bind index buffer.
        if let Some(index_buffer) = &self.index_buffer {
            match index_buffer {
                IndexBuffer::U16(index_buffer) => {
                    command_buffer_builder.bind_index_buffer(index_buffer.clone());
                },
                IndexBuffer::U32(index_buffer) => {
                    command_buffer_builder.bind_index_buffer(index_buffer.clone());
                }
            }
        }

        // bind vertex buffers.
        let vertex_buffers: Vec<_> = self.vertex_buffers.iter()
            .map(|buffer| buffer.buffer_access())
            .collect();
        if !vertex_buffers.is_empty() {
            command_buffer_builder.bind_vertex_buffers(0, vertex_buffers);
        }
    }

    /// Call the mesh's draw command.
    /// 
    /// # Unsafety
    /// You must to bind the mesh's buffer to the command buffer and then call the draw command.
    /// Otherwise, the mesh may not be drawn normally.
    /// 
    #[inline]
    pub unsafe fn draw<L, A: CommandBufferAllocator>(
        &self, 
        instance_count: u32,
        first_instance: u32,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<(), RuntimeError> {
        if self.index_buffer.is_some() {
            // draw with index buffer.
            command_buffer_builder.draw_indexed(
                self.index_count, 
                instance_count, 
                0, 
                0, 
                first_instance
            )
        }
        else {
            // draw vertex buffers.
            command_buffer_builder.draw(
                self.vertex_count, 
                instance_count,
                0, 
                first_instance
            )
        }.map_err(|e| err!("Vk Drawing Error: {}", e.to_string()))?;
        Ok(())
    }
}
