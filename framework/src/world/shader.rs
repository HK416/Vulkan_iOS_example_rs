use std::sync::Arc;
use std::collections::HashMap;

use vulkano::buffer::BufferContents;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::pipeline::{GraphicsPipeline, PipelineBindPoint, Pipeline};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::allocator::CommandBufferAllocator;

use crate::world::variable::ShaderVariableAbstract;
use crate::{err, error::RuntimeError};



pub struct GraphicsShader {
    pipeline: Arc<GraphicsPipeline>,
    variables: HashMap<u32, Arc<dyn ShaderVariableAbstract>>,
    descriptor_set: Option<Arc<PersistentDescriptorSet>>,
}

impl GraphicsShader {
    pub fn new<Iter>(
        pipeline: Arc<GraphicsPipeline>,
        allocator: &StandardDescriptorSetAllocator,
        variables: Iter,
    ) -> Result<Arc<Self>, RuntimeError> 
    where 
        Iter: IntoIterator<Item = Arc<dyn ShaderVariableAbstract>>,
        Iter::IntoIter: ExactSizeIterator,
    {
        let variables  = HashMap::from_iter(variables
            .into_iter()
            .enumerate()
            .map(|(bindings, variable)| {
                (bindings as u32, variable)
            })
        );

        let descriptor_set = if !variables.is_empty() {
            let descriptor_writes: Vec<_> = variables
                .iter()
                .map(|(&binding, variable)| {
                    variable.write_descriptor(binding)
                })
                .collect();

            let layout = pipeline.layout().set_layouts().get(0).unwrap().clone();
            let descriptor_set = match PersistentDescriptorSet::new(
                allocator, 
                layout, 
                descriptor_writes
            ) {
                Ok(it) => it,
                Err(e) => return Err(err!("Descriptor set creation failed: {}", e.to_string()))
            };

            Some(descriptor_set)
        }
        else {
            None
        };
        
        Ok(Arc::new(Self {
            pipeline,
            variables,
            descriptor_set
        }))
    }

    #[inline]
    pub unsafe fn bind_pipeline<L, A: CommandBufferAllocator>(
        &self, 
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) {
        command_buffer_builder.bind_pipeline_graphics(self.pipeline.clone());
    }

    #[inline]
    pub unsafe fn bind_descriptor_set<L, A: CommandBufferAllocator>(
        &self,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) {
        if let Some(descriptor_set) = &self.descriptor_set {
            command_buffer_builder.bind_descriptor_sets(
                PipelineBindPoint::Graphics, 
                self.pipeline.layout().clone(), 
                0, 
                descriptor_set.clone()
            );
        }
    }

    #[inline]
    pub unsafe fn push_constants<Pc, L, A>(
        &self, 
        offset: u32,
        push_constants: Pc,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) 
    where
        Pc: BufferContents,
        A: CommandBufferAllocator,
    {
        command_buffer_builder.push_constants(
            self.pipeline.layout().clone(), 
            offset, 
            push_constants
        );
    }
}