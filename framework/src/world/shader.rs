use std::{collections::BTreeMap, hash::Hash};
use crate::renderer::*;
use super::model::DrawableModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelGraphicsShader {
    pipeline: Arc<GraphicsPipeline>,
}

impl ModelGraphicsShader {
    pub fn new<'vs, 'tcs, 'tes, 'gs, 'fs>(
        renderer: &Renderer,
        input_assembly_state: Option<InputAssemblyState>,
        vertex_input_state: Option<VertexInputState>,
        color_blend_state: Option<ColorBlendState>,
        depth_stencil_state: Option<DepthStencilState>,
        multisample_state: Option<MultisampleState>,
        rasterization_state: Option<RasterizationState>,
        viewport_state: Option<ViewportState>,
        discard_rectangle_state: Option<DiscardRectangleState>,
        tessellation_state: Option<TessellationState>,
        vertex_shader: EntryPoint<'vs>,
        tessellation_shader: Option<(EntryPoint<'tcs>, EntryPoint<'tes>)>,
        geometry_shader: Option<EntryPoint<'gs>>,
        fragment_shader: EntryPoint<'fs>,
    ) -> Result<Arc<Self>, RuntimeError> {
        let mut builder = GraphicsPipeline::start()
            .input_assembly_state(input_assembly_state.unwrap_or_default())
            .vertex_input_state(vertex_input_state.unwrap_or_default())
            .color_blend_state(color_blend_state.unwrap_or_default())
            .depth_stencil_state(depth_stencil_state.unwrap_or_default())
            .multisample_state(multisample_state.unwrap_or_default())
            .rasterization_state(rasterization_state.unwrap_or_default())
            .viewport_state(viewport_state.unwrap_or_default())
            .discard_rectangle_state(discard_rectangle_state.unwrap_or_default())
            .tessellation_state(tessellation_state.unwrap_or_default());

        builder = builder.vertex_shader(vertex_shader, ());

        if let Some((control_shader, evaluation_shaer)) = tessellation_shader {
            builder = builder.tessellation_shaders(
                control_shader, (), 
                evaluation_shaer, ()
            );
        }

        if let Some(shader) = geometry_shader {
            builder = builder.geometry_shader(shader, ());
        }

        builder = builder.fragment_shader(fragment_shader, ());

        Ok(Arc::new(Self { 
            pipeline: renderer.build_graphics_pipeline(builder)?
        }))
    }

    #[inline]
    pub fn create_descriptor_set(
        &self,
        renderer: &Renderer,
        descriptor_writes: impl IntoIterator<Item = WriteDescriptorSet>,
    ) -> Result<Arc<PersistentDescriptorSet>, RuntimeError> {
        let layout = self.pipeline.layout().set_layouts()[0].clone();
        renderer.create_descriptor_set(
            layout, 
            descriptor_writes
        )
    }

    #[inline]
    pub fn bind_pipeline_graphics<L, A: CommandBufferAllocator>(
        &self, builder: &mut AutoCommandBufferBuilder<L, A>
    ) {
        builder.bind_pipeline_graphics(self.pipeline.clone());
    }

    #[inline]
    pub fn bind_descriptor_sets<L, A: CommandBufferAllocator, S: DescriptorSetsCollection>(
        &self,
        first_set: u32,
        descriptor_sets: S,
        builder: &mut AutoCommandBufferBuilder<L, A>,
    ) {
        builder.bind_descriptor_sets(
            PipelineBindPoint::Graphics, 
            self.pipeline.layout().clone(), 
            first_set, 
            descriptor_sets
        );
    }

    #[inline]
    pub fn push_constants<L, A: CommandBufferAllocator, Pc: BufferContents>(
        &self, 
        offset: u32,
        push_constants: Pc,
        builder: &mut AutoCommandBufferBuilder<L, A>
    ) {
        builder.push_constants(
            self.pipeline.layout().clone(), 
            offset, 
            push_constants
        );
    }

    pub fn draw_models<'a>(
        &self,
        models: impl Iterator<Item = &'a Arc<Mutex<dyn DrawableModel>>>,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
    ) -> Result<(), RuntimeError>{
        for model in models {
            let mut guard = model.lock().unwrap();
            guard.prepare_drawing(self, builder)?;
            guard.draw(self, builder)?;
        }
        Ok(())
    }
}
