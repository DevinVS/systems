use crate::graphics::atlas::Atlas;

use std::sync::Arc;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;

use crate::graphics::camera::Camera;

use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::RenderPassBeginInfo;
use vulkano::buffer::TypedBufferAccess;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::pipeline::Pipeline;
use std::convert::TryFrom;
use vulkano::swapchain::SwapchainCreateInfo;
use vulkano::shader::ShaderModule;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::cpu_pool::CpuBufferPoolChunk;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder,
    CommandBufferUsage,
    SubpassContents
};
use vulkano::device::{
    Device,
    DeviceExtensions,
    Queue,
    DeviceCreateInfo,
    QueueCreateInfo
};
use vulkano::device::physical::PhysicalDevice;
use vulkano::format::Format;
use vulkano::image::{
    ImageAccess,
    ImageDimensions,
    ImageUsage,
    ImmutableImage,
    MipmapsCount,
    SwapchainImage
};
use vulkano::image::view::ImageView;
use vulkano::instance::Instance;
use vulkano::memory::pool::StdMemoryPool;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::viewport::Scissor;
use vulkano::render_pass::{
    Framebuffer,
    RenderPass,
    Subpass
};
use vulkano::sampler::{
    Filter,
    Sampler,
    SamplerAddressMode,SamplerCreateInfo 
};
use vulkano::swapchain::{
    AcquireError,
    Surface,
    Swapchain,
    SwapchainCreationError
};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::swapchain;
use vulkano::instance::InstanceCreateInfo;

use vulkano_win::VkSurfaceBuild;

use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::Fullscreen;
use winit::window::Window;
use winit::window::WindowBuilder;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2]
}
vulkano::impl_vertex!(Vertex, position, tex_coords);

pub struct VulkanState {
    _instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,

    surface: Arc<Surface<Window>>,
    dimensions: [u32; 2],
    swapchain: Arc<Swapchain<Window>>,
    _images: Vec<Arc<SwapchainImage<Window>>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    viewport: Viewport,
    scissor: Scissor,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    _vs: Arc<ShaderModule>,
    _fs: Arc<ShaderModule>,

    vertex_buffer_pool: CpuBufferPool<Vertex>,
    index_buffer_pool: CpuBufferPool<u16>,
    uniform_buffer_pool: CpuBufferPool<vs::ty::Data>,

    vertex_buffer: Arc<CpuBufferPoolChunk<Vertex, Arc<StdMemoryPool>>>,
    index_buffer: Arc<CpuBufferPoolChunk<u16, Arc<StdMemoryPool>>>,

    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,

    atlas: Arc<ImageView<ImmutableImage>>,
    sampler: Arc<Sampler>,
}

impl VulkanState {
    pub fn new(event_loop: &EventLoop<()>, atlas: &Atlas) -> VulkanState {
        // Required extensions for rendering to a window
        let required_extensions = vulkano_win::required_extensions();

        // Create vulkan instance with required extensions
        let instance = Instance::new(
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                enumerate_portability: true,
                ..Default::default()
            }
        ).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let surface = WindowBuilder::new()
            .with_always_on_top(true)
            .with_decorations(false)
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .with_resizable(false)
            .with_transparent(true)
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        let (physical, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| {
                p.supported_extensions().is_superset_of(&device_extensions)
            })
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| {
                        q.supports_graphics() && q.supports_surface(&surface).unwrap_or(false)
                    })
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| {
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                }
            })
            .expect("No suitable physical device found");

        let (device, mut queues) = Device::new(
            physical,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                ..Default::default()
            }
        ).unwrap();

        // The only queue we need right now is for rendering, may need transfer queue later
        let queue = queues.next().unwrap();

        // Load shaders
        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        // Create swapchain
        let (swapchain, images) = {
            let caps = physical.surface_capabilities(&surface, Default::default()).unwrap();
            //let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

            // Internal format for images
            let format = Some(
                physical
                    .surface_formats(&surface, Default::default())
                    .unwrap()[0]
                    .0,
            );

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: caps.min_image_count,
                    image_format: format,
                    image_extent: surface.window().inner_size().into(),
                    image_usage: ImageUsage::color_attachment(),
                    composite_alpha: caps
                        .supported_composite_alpha
                        .iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                }
            ).unwrap()
        };

        let dimensions: [u32; 2] = surface.window().inner_size().into();

        // We now create a buffer that will store the shape of our square
        let vertex_buffer_pool = CpuBufferPool::vertex_buffer(device.clone());
        let index_buffer_pool = CpuBufferPool::new(device.clone(), BufferUsage::index_buffer());
        let uniform_buffer_pool = CpuBufferPool::uniform_buffer(device.clone());

        let vertex_buffer = vertex_buffer_pool.chunk([]).unwrap();
        let index_buffer = index_buffer_pool.chunk([]).unwrap();

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap();

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let mut scissor = Scissor {
            origin: [0, 0],
            dimensions: [0, 0],
        };

        let (atlas_tex, atlas_fut) = {
            let (info, image_data) = atlas.image_data();

            let dimensions = ImageDimensions::Dim2d {
                width: info.width,
                height: info.height,
                array_layers: 1
            };
    
            let format = Format::R8G8B8A8_SRGB;
    
            let (image, future) = ImmutableImage::from_iter(
                image_data.iter().cloned(),
                dimensions,
                MipmapsCount::One,
                format,
                queue.clone()
            ).unwrap();
    
            (ImageView::new_default(image).unwrap(), future)
        };

        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                address_mode: [SamplerAddressMode::Repeat; 3],
                ..Default::default()
            }
        ).unwrap();

        let pipeline = GraphicsPipeline::start()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .input_assembly_state(InputAssemblyState::new())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_dynamic(1))
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .color_blend_state(ColorBlendState::default().blend_alpha())
            .build(device.clone())
            .unwrap();

        // Actual framebuffers to draw to
        let framebuffers = VulkanState::window_size_dependent_setup(&images, render_pass.clone(), &mut viewport, &mut scissor, None);
        let previous_frame_end = Some(atlas_fut.boxed());

        VulkanState {
            _instance: instance,
            device,
            queue,
            dimensions,
            surface,
            swapchain,
            _images: images,
            framebuffers,
            render_pass,
            pipeline,
            _vs: vs,
            _fs: fs,
            vertex_buffer_pool,
            index_buffer_pool,
            uniform_buffer_pool,
            viewport,
            scissor,
            vertex_buffer,
            index_buffer,
            recreate_swapchain: false,
            previous_frame_end,
            atlas: atlas_tex,
            sampler,
        }
    }

    pub fn draw(&mut self, camera: &Camera) {
        let uniform_buffer_subbuffer = {
            let worldview = camera.matrix();
            let uniform_data = vs::ty::Data {
                worldview: worldview.into(),
            };
            self.uniform_buffer_pool.next(uniform_data).unwrap()
        };

        // Descriptor set
        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer),
                WriteDescriptorSet::image_view_sampler(1, self.atlas.clone(), self.sampler.clone())
            ]
        ).unwrap();

        // Acquire image from swapchain
        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
    
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(self.framebuffers[image_num].clone())
                },
                SubpassContents::Inline,
            ).unwrap()
            .set_viewport(0, [self.viewport.clone()])
            .set_scissor(0, [self.scissor.clone()])
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                set.clone()
            )
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .bind_index_buffer(self.index_buffer.clone())
            .draw_indexed(self.index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();
        let command_buffer = builder.build().unwrap();

        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();
        
        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(sync::FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        }
    }

    pub fn transfer_object_data<I, J>(&mut self, vertices: I, indices: J)
    where
        I: IntoIterator<Item = Vertex>,
        I::IntoIter: ExactSizeIterator,
        J: IntoIterator<Item = u16>,
        J::IntoIter: ExactSizeIterator,
    {
        self.vertex_buffer = self.vertex_buffer_pool.chunk(vertices).unwrap();
        self.index_buffer = self.index_buffer_pool.chunk(indices).unwrap();
    }

    pub fn recreate_swapchain(&mut self, camera: &Camera) {
        // Get the new dimensions of the window.
        self.dimensions = self.surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match self.swapchain.recreate(SwapchainCreateInfo {
                image_extent: self.dimensions.into(),
                ..self.swapchain.create_info()
            }) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

        self.swapchain = new_swapchain;
        // Because framebuffers contains an Arc on the old swapchain, we need to
        // recreate framebuffers as well.
        self.framebuffers = VulkanState::window_size_dependent_setup(&new_images,self.render_pass.clone(), &mut self.viewport, &mut self.scissor, Some(camera));
        self.recreate_swapchain = false;
    }

    fn window_size_dependent_setup(
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<RenderPass>,
        viewport: &mut Viewport,
        scissor: &mut Scissor,
        camera: Option<&Camera>,
    ) -> Vec<Arc<Framebuffer>> {
        let dimensions: [u32; 2] = images[0].dimensions().width_height();
        viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

        *scissor = if let Some(camera) = camera {
            let (origin, dimensions) = camera.viewport();
            Scissor {
                origin,
                dimensions
            }
        } else {
            Scissor {
                origin: [0,0],
                dimensions
            }
        };

        images.iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    }
                ).unwrap()
            }).collect::<Vec<_>>()
    }

    pub fn logical_size(&self) -> LogicalSize<u32> {
        self.surface.window().inner_size().to_logical(self.surface.window().scale_factor())
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/rect.vs",
        types_meta: { use bytemuck::{Pod, Zeroable}; #[derive(Copy, Clone, Pod, Zeroable)] },
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/graphics/shaders/rect.fs"
    }
}
