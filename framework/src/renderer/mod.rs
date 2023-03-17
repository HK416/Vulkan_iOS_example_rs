mod utility;
mod platform;

use std::fs::File;
use std::io::Read;
use std::ops::RangeInclusive;
pub use std::path::{Path, PathBuf};
pub use std::sync::{Arc, Weak, Mutex};
pub use std::collections::HashMap;

pub use vulkano::*;
pub use vulkano::instance::*;
pub use vulkano::instance::debug::*;
pub use vulkano::device::*;
pub use vulkano::device::physical::*;
pub use vulkano::swapchain::*;
pub use vulkano::swapchain::display::*;
pub use vulkano::buffer::*;
pub use vulkano::buffer::view::*;
pub use vulkano::buffer::cpu_pool::*;
pub use vulkano::buffer::cpu_access::*;
pub use vulkano::buffer::device_local::*;
pub use vulkano::image::*;
pub use vulkano::image::view::*;
pub use vulkano::render_pass::*;
pub use vulkano::memory::*;
pub use vulkano::memory::allocator::*;
pub use vulkano::command_buffer::*;
pub use vulkano::command_buffer::pool::*;
pub use vulkano::command_buffer::synced::*;
pub use vulkano::command_buffer::allocator::*;
pub use vulkano::pipeline::*;
pub use vulkano::pipeline::cache::*;
pub use vulkano::pipeline::layout::*;
pub use vulkano::pipeline::compute::*;
pub use vulkano::pipeline::graphics::*;
pub use vulkano::pipeline::graphics::viewport::*;
pub use vulkano::pipeline::graphics::render_pass::*;
pub use vulkano::pipeline::graphics::input_assembly::*;
pub use vulkano::pipeline::graphics::vertex_input::*;
pub use vulkano::pipeline::graphics::color_blend::*;
pub use vulkano::pipeline::graphics::depth_stencil::*;
pub use vulkano::pipeline::graphics::rasterization::*;
pub use vulkano::pipeline::graphics::multisample::*;
pub use vulkano::pipeline::graphics::tessellation::*;
pub use vulkano::pipeline::graphics::discard_rectangle::*;
pub use vulkano::descriptor_set::*;
pub use vulkano::descriptor_set::pool::*;
pub use vulkano::descriptor_set::layout::*;
pub use vulkano::descriptor_set::allocator::*;
pub use vulkano::descriptor_set::persistent::*;
pub use vulkano::shader::*;
pub use vulkano::shader::spirv::*;
pub use vulkano::shader::reflect::*;
pub use vulkano::sampler::*;
pub use vulkano::sampler::ycbcr::*;
pub use vulkano::half::prelude::*;
pub use vulkano::range_set::*;
pub use vulkano::format::*;
pub use vulkano::sync::*;


pub use crate::{err, error::RuntimeError};
pub use self::platform::AppHandle;
pub use self::utility::{rgb, rgba};


use utility::*;
use platform::*;

pub struct Renderer {
    handle: AppHandle,
    assets_dir: PathBuf,
    scale_factor: f32,
    screen_size: [u32; 2],
    viewer_area: [i32; 4],
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    swapchain_image_views: Vec<Arc<ImageView<SwapchainImage>>>,
    depth_stencil_image_view: Arc<ImageView<AttachmentImage>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline_cache: Arc<PipelineCache>,
    std_mem_allocator: Arc<StandardMemoryAllocator>,
    cmd_buf_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    max_frame_in_flight: u32,
    current_frame: u32,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swapchain: bool,
}

impl Renderer {
    pub fn new(
        handle: AppHandle, 
        assets_dir: &Path,
        scale_factor: f32,
        screen_size: [u32; 2],
        viewer_area: [i32; 4],
    ) -> Result<Self, RuntimeError> {
        let size = screen_size
            .map(|n| n as f32 * scale_factor)
            .map(|n| n as u32);

        let library = load_vulkan_library()?;
        let instance = create_vulkan_instance(&library)?;
        let surface = create_vulkan_surface(&handle, &instance)?;
        let (device, queue) = create_vulkan_device_and_queue(&instance, &surface)?;
        let (swapchain, swapchain_image_views, max_frame_in_flight) = create_vulkan_swapchain(size, &device, &surface)?;
        let std_mem_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let cmd_buf_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(device.clone()));
        let depth_stencil_image_view = create_depth_stencil(size, &std_mem_allocator, &device)?;
        let render_pass = create_vulkan_render_pass(&device, Some(swapchain.image_format()), depth_stencil_image_view.format())?;
        let framebuffers = create_vulkan_framebuffers(size, &render_pass, &swapchain_image_views, &depth_stencil_image_view)?;
        let previous_frame_end = Some(vulkano::sync::now(device.clone()).boxed());

        let pipeline_cache = PipelineCache::empty(device.clone())
            .map_err(|e| err!("Vk Pipeline Cache Creation Error: {}", e.to_string()))?;

        Ok(Self { 
            handle,
            assets_dir: assets_dir.to_path_buf(),
            scale_factor,
            screen_size,
            viewer_area,
            instance, 
            surface, 
            device,
            queue,
            swapchain,
            swapchain_image_views,
            depth_stencil_image_view,
            render_pass,
            framebuffers,
            pipeline_cache,
            std_mem_allocator,
            cmd_buf_allocator,
            descriptor_set_allocator,
            max_frame_in_flight,
            current_frame: 0,
            previous_frame_end,
            recreate_swapchain: false,
        })
    }

    #[inline]
    pub fn ref_assets_dir(&self) -> &Path {
        &self.assets_dir
    }

    #[inline]
    pub fn get_swapchain_image_extent(&self) -> [u32; 2] {
        self.swapchain.image_extent()
    }

    #[inline]
    pub fn get_max_frame_in_flight(&self) -> u32 {
        self.max_frame_in_flight
    }

    #[inline]
    pub fn get_current_frame_idx(&self) -> u32 {
        self.current_frame
    }

    #[inline]
    pub fn get_device_properties(&self) -> &Properties {
        self.device.physical_device().properties()
    }

    #[inline]
    pub fn wait_for_next_frame(&mut self) -> Result<Option<SwapchainAcquireFuture>, RuntimeError> {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            let (new_swapchain, new_swapchain_image_views, new_framebuffers) = 
                recreate_vulkan_swapchain(
                self.screen_size, 
                &self.swapchain, 
                &self.render_pass, 
                &self.depth_stencil_image_view
            )?;

            self.swapchain = new_swapchain;
            self.swapchain_image_views = new_swapchain_image_views;
            self.framebuffers = new_framebuffers;

            self.recreate_swapchain = false;

            // DEBUG --------------------------------------------------------------------
            println!("swapchain recreated. (size:{:?})", self.screen_size);
        }

        let (image_index, suboptimal, acquire_future) = 
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(item) => item,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(None);
                },
                Err(e) => {
                    return Err(err!("Vk Presentation Error:{}", e.to_string()));
                }
        };
        self.current_frame = image_index;

        if suboptimal {
            self.recreate_swapchain = true;
        }

        Ok(Some(acquire_future))
    }

    #[inline]
    pub fn begin_render_pass<A>(
        &self,
        clear_values: Vec<Option<ClearValue>>,
        render_area_offset: Option<[u32; 2]>,
        render_area_extent: Option<[u32; 2]>,
        contents: SubpassContents,
        mut command_buffer_builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, A>
    ) -> Result<CmdBufBeginRenderPassGuard<A>, RuntimeError> 
    where A: CommandBufferAllocator {
        command_buffer_builder.begin_render_pass(
            RenderPassBeginInfo {
                clear_values,
                render_pass: self.render_pass.clone(),
                render_area_offset: render_area_offset.unwrap_or_default(),
                render_area_extent: render_area_extent.unwrap_or(self.framebuffers[self.current_frame as usize].extent()),
                ..RenderPassBeginInfo::framebuffer(self.framebuffers[self.current_frame as usize].clone())
            }, 
            contents
        ).map_err(|e| err!("Vk Begin Render Pass Error: {}", e.to_string()))?;

        Ok(CmdBufBeginRenderPassGuard { 
            render_pass: self.render_pass.clone(),
            framebuffers: self.framebuffers[self.current_frame as usize].clone(),
            command_buffer_builder,
        })
    }

    #[inline]
    pub fn queue_submit_and_present<A>(
        &mut self,
        acquire_future: SwapchainAcquireFuture,
        command_buffer: PrimaryAutoCommandBuffer<A>
    ) -> Result<(), RuntimeError> 
    where A: CommandBufferAlloc  {
        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .map_err(|e| err!("Vk Comand Buffer Execution Error: {}", e.to_string()))?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(), 
                    self.current_frame
                )
            ).then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            },
            Err(FlushError::OutOfDate) => {
                println!("flush error!");
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(now(self.device.clone()).boxed());
            },
            Err(e) => {
                return Err(err!("Vk Presentation Error: {}", e.to_string()))
            }
        }
        Ok(())
    }

    #[inline]
    pub fn queue_submit(
        &self,
        builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>
    ) -> Result<CommandBufferExecFuture<NowFuture>, RuntimeError> {
        let command_buffer = builder.build()
            .map_err(|e| err!("Vk Command Buffer Building Error: {}", e.to_string()))?;
        
        command_buffer.execute(self.queue.clone())
            .map_err(|e| err!("Vk Command Buffer Execution Error: {}", e.to_string()))
    }

    #[inline]
    pub fn primary_command_buffer(
        &self, 
        usage: Option<CommandBufferUsage>
    ) -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, RuntimeError> {
        AutoCommandBufferBuilder::primary(
            self.cmd_buf_allocator.as_ref(), 
            self.queue.queue_family_index(), 
            usage.unwrap_or(CommandBufferUsage::OneTimeSubmit)
        ).map_err(|e| err!("Vk Command Buffer Creation Error: {}", e.to_string()))
    }

    #[inline]
    pub fn secondary_command_buffer(
        &self, 
        usage: Option<CommandBufferUsage>,
        inheritance_info: Option<CommandBufferInheritanceInfo>
    ) -> Result<AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, RuntimeError> {
        AutoCommandBufferBuilder::secondary(
            self.cmd_buf_allocator.as_ref(),
            self.queue.queue_family_index(), 
            usage.unwrap_or(CommandBufferUsage::OneTimeSubmit),
            inheritance_info.unwrap_or_default()
        ).map_err(|e| err!("Vk Command Buffer Creation Error: {}", e.to_string()))
    }

    #[inline]
    pub fn create_accessible_buffer_from_data<T>(
        &self, 
        usage: BufferUsage,
        host_cached: bool,
        data: T,
    ) -> Result<Arc<CpuAccessibleBuffer<T>>, RuntimeError> 
    where T: BufferContents {
        CpuAccessibleBuffer::from_data(
            &self.std_mem_allocator, 
            usage, 
            host_cached, 
            data
        ).map_err(|e| err!("Vk Accessible Buffer Creation Error: {}", e.to_string()))
    }

    #[inline]
    pub fn create_accessible_buffer_from_iter<T, D>(
        &self, 
        usage: BufferUsage,
        host_cached: bool, 
        data: D,
    ) -> Result<Arc<CpuAccessibleBuffer<[T]>>, RuntimeError> 
    where [T]: BufferContents, D: IntoIterator<Item = T>, D::IntoIter: ExactSizeIterator {
        CpuAccessibleBuffer::from_iter(
            &self.std_mem_allocator, 
            usage, 
            host_cached, 
            data
        ).map_err(|e| err!("Vk Accessible Buffer Creation Error: {}", e.to_string()))
    }

    #[inline]
    pub fn create_device_local_buffer_from_buffer<T, B, L, A>(
        &self,
        source: Arc<B>,
        usage: BufferUsage,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<DeviceLocalBuffer<T>>, RuntimeError> 
    where T: BufferContents + ?Sized, B: TypedBufferAccess<Content = T> + 'static, A: CommandBufferAllocator {
        DeviceLocalBuffer::from_buffer(
            &self.std_mem_allocator, 
            source, 
            usage, 
            command_buffer_builder
        ).map_err(|e| err!("Vk Local Buffer Creation Error: {}", e.to_string()))
    }

    #[inline]
    pub fn create_device_local_buffer_from_data<T, L, A>(
        &self,
        data: T,
        usage: BufferUsage,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<DeviceLocalBuffer<T>>, RuntimeError> 
    where T: BufferContents, A: CommandBufferAllocator {
        DeviceLocalBuffer::from_data(
            self.std_mem_allocator.as_ref(), 
            data, 
            usage, 
            command_buffer_builder
        ).map_err(|e| err!("Vk Local Buffer Creation Error: {}", e.to_string()))        
    }

    #[inline]
    pub fn create_device_local_buffer_from_iter<T, D, L, A>(
        &self,
        data: D,
        usage: BufferUsage,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L, A>
    ) -> Result<Arc<DeviceLocalBuffer<[T]>>, RuntimeError>
    where [T]: BufferContents, D: IntoIterator<Item = T>, D::IntoIter: ExactSizeIterator, A: CommandBufferAllocator {
        DeviceLocalBuffer::from_iter(
            self.std_mem_allocator.as_ref(), 
            data, 
            usage, 
            command_buffer_builder
        ).map_err(|e| err!("Vk Local Buffer Creation Error: {}", e.to_string()))
    }

    #[inline]
    pub fn create_cpu_buffer_pool<T>(
        &self,
        buffer_usage: BufferUsage,
        memory_usage: MemoryUsage,
    ) -> CpuBufferPool<T>
    where [T]: BufferContents {
        CpuBufferPool::new(
            self.std_mem_allocator.clone(),
            buffer_usage, 
            memory_usage
        )
    }

    #[inline]
    pub fn load_shader_from_spv_file(
        &self,
        file_path: &Path
    ) -> Result<Arc<ShaderModule>, RuntimeError> {
        let mut f = File::open(file_path)
            .map_err(|e| err!("Failed to file: {}", e.to_string()))?;
        
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|e| err!("Failed to read file: {}", e.to_string()))?;

        unsafe { 
            ShaderModule::from_bytes(self.device.clone(), &buf) 
        }.map_err(|e| err!("Vk Shader Module Creation Error: {}", e.to_string()))
    }

    #[inline]
    pub fn create_descriptor_set(
        &self,
        layout: Arc<DescriptorSetLayout>,
        descriptor_writes: impl IntoIterator<Item = WriteDescriptorSet>
    ) -> Result<Arc<PersistentDescriptorSet>, RuntimeError> {
        PersistentDescriptorSet::new(
            &self.descriptor_set_allocator, 
            layout, 
            descriptor_writes
        ).map_err(|e| err!("Vk Descriptor Set Allocation Error: {}", e.to_string()))
    }

    #[inline]
    pub fn build_graphics_pipeline<'vs, 'tcs, 'tes, 'gs, 'fs>(
        &self, 
        builder: GraphicsPipelineBuilder<'vs, 'tcs, 'tes, 'gs, 'fs, VertexInputState, (), (), (), (), ()>,
    ) -> Result<Arc<GraphicsPipeline>, RuntimeError> {
        builder.render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
            .build_with_cache(self.pipeline_cache.clone())
            .build(self.device.clone())
            .map_err(|e| err!("Vk Graphics Pipeline Creation Error: {}", e.to_string()))
    }
}

pub struct CmdBufBeginRenderPassGuard<A = StandardCommandBufferAllocator> 
where A: CommandBufferAllocator {
    render_pass: Arc<RenderPass>,
    framebuffers: Arc<Framebuffer>,
    command_buffer_builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, A>,
}

impl<A> CmdBufBeginRenderPassGuard<A> 
where A: CommandBufferAllocator {
    #[inline]
    pub fn inheritance_info(&self) -> CommandBufferInheritanceInfo {
        CommandBufferInheritanceInfo {
            render_pass: Some(
                CommandBufferInheritanceRenderPassType::BeginRenderPass(
                    CommandBufferInheritanceRenderPassInfo { 
                        subpass: Subpass::from(self.render_pass.clone(), 0).unwrap(),
                        framebuffer: Some(self.framebuffers.clone())
                    }
                )
            ),
            ..Default::default()
        }
    }

    #[inline]
    pub fn end_render_pass(mut self) -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, A>, RenderPassError> {
        self.command_buffer_builder.end_render_pass()?;
        Ok(self.command_buffer_builder)
    }

    #[inline]
    pub fn bind_descriptor_sets<S>(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        pipeline_layout: Arc<PipelineLayout>,
        first_set: u32,
        descriptor_sets: S
    ) -> &mut Self 
    where S: DescriptorSetsCollection {
        self.command_buffer_builder.bind_descriptor_sets(
            pipeline_bind_point, 
            pipeline_layout, 
            first_set, 
            descriptor_sets
        );
        return self;
    }

    #[inline]
    pub fn bind_index_buffer<Ib, I>(
        &mut self,
        index_buffer: Arc<Ib>
    ) -> &mut Self 
    where Ib: TypedBufferAccess<Content = [I]> + 'static, I: Index + 'static {
        self.command_buffer_builder.bind_index_buffer(index_buffer);
        return self;
    }
    
    #[inline]
    pub fn bind_pipeline_graphics(
        &mut self,
        pipeline: Arc<GraphicsPipeline>
    ) -> &mut Self {
        self.command_buffer_builder.bind_pipeline_graphics(pipeline);
        return self;
    }

    #[inline]
    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        vertex_buffers: impl VertexBuffersCollection
    ) -> &mut Self {
        self.command_buffer_builder.bind_vertex_buffers(first_binding, vertex_buffers);
        return self;
    }

    #[inline]
    pub fn blit_image(
        &mut self,
        blit_image_info: BlitImageInfo
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.blit_image(blit_image_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn clear_attachments(
        &mut self,
        attachments: impl IntoIterator<Item = ClearAttachment>,
        rects: impl IntoIterator<Item = ClearRect>
    ) -> Result<&mut Self, RenderPassError> {
        self.command_buffer_builder.clear_attachments(attachments, rects)?;
        return Ok(self);
    }

    #[inline]
    pub fn clear_color_image(
        &mut self,
        clear_info: ClearColorImageInfo,
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.clear_color_image(clear_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn clear_depth_stencil_image(
        &mut self,
        clear_info: ClearDepthStencilImageInfo
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.clear_depth_stencil_image(clear_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn copy_buffer(
        &mut self,
        copy_buffer_info: impl Into<CopyBufferInfo>
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.copy_buffer(copy_buffer_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn copy_buffer_to_image(
        &mut self,
        copy_buffer_to_image_info: CopyBufferToImageInfo
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.copy_buffer_to_image(copy_buffer_to_image_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn copy_image(
        &mut self,
        copy_image_info: CopyImageInfo
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.copy_image(copy_image_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn copy_image_to_buffer(
        &mut self,
        copy_image_to_buffer_info: CopyImageToBufferInfo
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.copy_image_to_buffer(copy_image_to_buffer_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32
    ) -> Result<&mut Self, PipelineExecutionError> {
        self.command_buffer_builder.draw(
            vertex_count, 
            instance_count, 
            first_vertex, 
            first_instance
        )?;
        return Ok(self);
    }

    #[inline]
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32
    ) -> Result<&mut Self, PipelineExecutionError> {
        self.command_buffer_builder.draw_indexed(
            index_count, 
            instance_count, 
            first_index, 
            vertex_offset, 
            first_instance
        )?;
        return Ok(self);
    }

    #[inline]
    pub fn draw_indexed_indirect<Inb>(
        &mut self,
        indirect_buffer: Arc<Inb>
    ) -> Result<&mut Self, PipelineExecutionError> 
    where Inb: TypedBufferAccess<Content = [DrawIndexedIndirectCommand]> + 'static {
        self.command_buffer_builder.draw_indexed_indirect(indirect_buffer)?;
        return Ok(self);
    }

    #[inline]
    pub fn draw_indirect<Inb>(
        &mut self,
        indirect_buffer: Arc<Inb>
    ) -> Result<&mut Self, PipelineExecutionError> 
    where Inb: TypedBufferAccess<Content = [DrawIndirectCommand]> + Send + Sync + 'static {
        self.command_buffer_builder.draw_indirect(indirect_buffer)?;
        return Ok(self);
    }

    #[inline]
    pub fn execute_commands<C>(
        &mut self,
        command_buffer: C
    ) -> Result<&mut Self, ExecuteCommandsError> 
    where C: SecondaryCommandBufferAbstract + 'static {
        self.command_buffer_builder.execute_commands(command_buffer)?;
        return Ok(self);
    }

    #[inline]
    pub fn execute_commands_from_vec<C>(
        &mut self,
        command_buffers: Vec<C>
    ) -> Result<&mut Self, ExecuteCommandsError> 
    where C: SecondaryCommandBufferAbstract + 'static {
        self.command_buffer_builder.execute_commands_from_vec(command_buffers)?;
        return Ok(self);
    }

    #[inline]
    pub fn fill_buffer(
        &mut self,
        fill_buffer_info: FillBufferInfo
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.fill_buffer(fill_buffer_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn next_subpass(
        &mut self,
        contents: SubpassContents
    ) -> Result<&mut Self, RenderPassError> {
        self.command_buffer_builder.next_subpass(contents)?;
        return Ok(self);
    }

    #[inline]
    pub fn push_constants<Pc>(
        &mut self,
        pipeline_layout: Arc<PipelineLayout>,
        offset: u32,
        push_constants: Pc
    ) -> &mut Self 
    where Pc: BufferContents {
        self.command_buffer_builder.push_constants(
            pipeline_layout, 
            offset, 
            push_constants
        );
        return self;
    }

    #[inline]
    pub fn push_descriptor_set(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        pipeline_layout: Arc<PipelineLayout>,
        set_num: u32,
        descriptor_writes: impl IntoIterator<Item = WriteDescriptorSet>
    ) -> &mut Self {
        self.command_buffer_builder.push_descriptor_set(
            pipeline_bind_point, 
            pipeline_layout, 
            set_num, 
            descriptor_writes
        );
        return self;
    }

    #[inline]
    pub fn resolve_image(
        &mut self,
        resolve_image_info: ResolveImageInfo
    ) -> Result<&mut Self, CopyError> {
        self.command_buffer_builder.resolve_image(resolve_image_info)?;
        return Ok(self);
    }

    #[inline]
    pub fn set_blend_constants(
        &mut self,
        constants: [f32; 4]
    ) -> &mut Self {
        self.command_buffer_builder.set_blend_constants(constants);
        return self;
    }

    #[inline]
    pub fn set_color_wrtie_enable<I>(
        &mut self,
        enables: I
    ) -> &mut Self 
    where I: IntoIterator<Item = bool>, I::IntoIter: ExactSizeIterator {
        self.command_buffer_builder.set_color_write_enable(enables);
        return self;
    }

    #[inline]
    pub fn set_cull_mode(
        &mut self,
        cull_mode: CullMode
    ) -> &mut Self {
        self.command_buffer_builder.set_cull_mode(cull_mode);
        return self;
    }

    #[inline]
    pub fn set_depth_bias(
        &mut self,
        constant_factor: f32,
        clamp: f32,
        slope_factor: f32
    ) -> &mut Self {
        self.command_buffer_builder.set_depth_bias(
            constant_factor, 
            clamp, 
            slope_factor
        );
        return self;
    }

    #[inline]
    pub fn set_depth_bias_enable(
        &mut self,
        enable: bool
    ) -> &mut Self {
        self.command_buffer_builder.set_depth_bias_enable(enable);
        return self;
    }

    #[inline]
    pub fn set_depth_bounds(
        &mut self,
        bounds: RangeInclusive<f32>
    ) -> &mut Self {
        self.command_buffer_builder.set_depth_bounds(bounds);
        return self;
    }

    #[inline]
    pub fn set_depth_bounds_test_enable(
        &mut self,
        enable: bool
    ) -> &mut Self {
        self.command_buffer_builder.set_depth_bounds_test_enable(enable);
        return self;
    }

    #[inline]
    pub fn set_depth_compare_op(
        &mut self,
        compare_op: CompareOp
    ) -> &mut Self {
        self.command_buffer_builder.set_depth_compare_op(compare_op);
        return self;    
    }

    #[inline]
    pub fn set_depth_test_enable(
        &mut self,
        enable: bool
    ) -> &mut Self {
        self.command_buffer_builder.set_depth_test_enable(enable);
        return self;
    }

    #[inline]
    pub fn set_depth_write_enable(
        &mut self,
        enable: bool
    ) -> &mut Self {
        self.command_buffer_builder.set_depth_write_enable(enable);
        return self;
    }

    #[inline]
    pub fn set_discard_rectangle(
        &mut self,
        first_rectangle: u32,
        rectangles: impl IntoIterator<Item = Scissor>
    ) -> &mut Self {
        self.command_buffer_builder.set_discard_rectangle(
            first_rectangle, 
            rectangles
        );
        return self;
    }

    #[inline]
    pub fn set_front_face(
        &mut self,
        face: FrontFace
    ) -> &mut Self {
        self.command_buffer_builder.set_front_face(face);
        return self;
    }

    #[inline]
    pub fn set_line_stipple(
        &mut self,
        factor: u32,
        pattern: u16
    ) -> &mut Self {
        self.command_buffer_builder.set_line_stipple(
            factor, 
            pattern
        );
        return self;
    }

    #[inline]
    pub fn set_line_width(
        &mut self,
        line_width: f32
    ) -> &mut Self {
        self.command_buffer_builder.set_line_width(line_width);
        return self;
    }

    #[inline]
    pub fn set_logic_op(
        &mut self,
        logic_op: LogicOp
    ) -> &mut Self {
        self.command_buffer_builder.set_logic_op(logic_op);
        return self;
    }

    #[inline]
    pub fn set_patch_control_points(
        &mut self,
        num: u32
    ) -> &mut Self {
        self.command_buffer_builder.set_patch_control_points(num);
        return self;
    }

    #[inline]
    pub fn set_primitive_restart_enable(
        &mut self,
        enable: bool
    ) -> &mut Self {
        self.command_buffer_builder.set_primitive_restart_enable(enable);
        return self;
    }

    #[inline]
    pub fn set_primitive_topology(
        &mut self,
        topology: PrimitiveTopology
    ) -> &mut Self {
        self.command_buffer_builder.set_primitive_topology(topology);
        return self;
    }

    #[inline]
    pub fn set_rasterizer_discard_enable(
        &mut self,
        enable: bool
    ) -> &mut Self {
        self.command_buffer_builder.set_rasterizer_discard_enable(enable);
        return self;
    }

    #[inline]
    pub fn set_scissor(
        &mut self,
        first_scissor: u32,
        scissors: impl IntoIterator<Item = Scissor>
    ) -> &mut Self {
        self.command_buffer_builder.set_scissor(first_scissor, scissors);
        return self;
    }

    #[inline]
    pub fn set_scissor_with_count(
        &mut self,
        scissors: impl IntoIterator<Item = Scissor>
    ) -> &mut Self {
        self.command_buffer_builder.set_scissor_with_count(scissors);
        return self;
    }

    #[inline]
    pub fn set_stencil_compare_mask(
        &mut self,
        faces: StencilFaces,
        compare_mask: u32
    ) -> &mut Self {
        self.command_buffer_builder.set_stencil_compare_mask(
            faces, 
            compare_mask
        );
        return self;
    }

    #[inline]
    pub fn set_stencil_op(
        &mut self,
        faces: StencilFaces,
        fail_op: StencilOp,
        pass_op: StencilOp,
        depth_fail_op: StencilOp,
        compare_op: CompareOp
    ) -> &mut Self {
        self.command_buffer_builder.set_stencil_op(
            faces, 
            fail_op, 
            pass_op, 
            depth_fail_op, 
            compare_op
        );
        return self;
    }

    #[inline]
    pub fn set_stencil_reference(
        &mut self,
        faces: StencilFaces,
        reference: u32
    ) -> &mut Self {
        self.command_buffer_builder.set_stencil_reference(
            faces, 
            reference
        );
        return self;
    }

    #[inline]
    pub fn set_stencil_test_enable(
        &mut self,
        enable: bool
    ) -> &mut Self {
        self.command_buffer_builder.set_stencil_test_enable(enable);
        return self;
    }

    #[inline]
    pub fn set_stencil_write_mask(
        &mut self,
        faces: StencilFaces,
        write_mask: u32
    ) -> &mut Self {
        self.command_buffer_builder.set_stencil_write_mask(
            faces, 
            write_mask
        );
        return self;
    }

    #[inline]
    pub fn set_viewport(
        &mut self,
        first_viewport: u32,
        viewports: impl IntoIterator<Item = Viewport>
    ) -> &mut Self {
        self.command_buffer_builder.set_viewport(
            first_viewport, 
            viewports
        );
        return self;
    }

    #[inline]
    pub fn set_viewport_with_count(
        &mut self,
        viewports: impl IntoIterator<Item = Viewport>
    ) -> &mut Self {
        self.command_buffer_builder.set_viewport_with_count(viewports);
        return self;
    }

    #[inline]
    pub fn state(&self) -> CommandBufferBuilderState<'_> {
        self.command_buffer_builder.state()
    }

    #[inline]
    pub fn update_buffer<B, D, Dd>(
        &mut self,
        data: Dd,
        dst_buffer: Arc<B>,
        dst_offset: DeviceSize
    ) -> Result<&mut Self, CopyError> 
    where 
        B: TypedBufferAccess<Content = D> + 'static, 
        D: BufferContents + ?Sized, 
        Dd: SafeDeref<Target = D> + Send + Sync + 'static {
        self.command_buffer_builder.update_buffer(
            data, 
            dst_buffer, 
            dst_offset
        )?;
        return Ok(self);
    }
}
