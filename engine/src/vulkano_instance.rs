extern crate winit;

use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::Surface;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::AcquireError;
use vulkano::image::swapchain::SwapchainImage;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;

use vulkano::device::Queue; 
use winit::Window;

use vulkano::framebuffer::Subpass;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::sync::FlushError;

use vulkano_win_frankenstein::vulkano_win_frankenstein;
use vulkano_win_frankenstein::vulkano_win_frankenstein::VkSurfaceBuild;


use std::sync::Arc;
use std::boxed::Box;
use std::time::Instant;
use std::mem;
use std::vec::Vec;
use std::option::Option;

#[derive(Debug, Clone)]
struct Vertex { position: [f32; 2] }
impl_vertex!(Vertex, position);

mod vs 
{
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
    #version 450
    layout(location = 0) in vec2 position;
    void main() 
    {
        gl_Position = vec4(position, 0.0, 1.0);
    }
    "]
    struct _Dummy;
}

mod fs 
{
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
    #version 450
    layout(location = 0) out vec4 f_color;
    void main() 
    {
        f_color = vec4(0.0, 0.0, 1.0, 1.0);
    }
    "]
    struct _Dummy;
}


pub struct VulkanoInstance
{
    device : Arc<Device>,
    start : Instant,
    previous_frame_time : Instant,
    previous_frame_end_future : Box<GpuFuture>,
    window : Arc<Surface<Window>>,
    swapchain : Arc<Swapchain<Window>>,
    images : Vec<Arc<SwapchainImage<Window>>>,
    render_pass : Arc<RenderPassAbstract + Send + Sync>,
    graphics_queue : Arc<Queue>,
    dimensions : [u32; 2],
    vertex_shader : vs::Shader,
    fragment_shader : fs::Shader,
    image_index : usize,
    acquire_future : Option<SwapchainAcquireFuture<Window>>,
    command_buffer : Option<AutoCommandBuffer>,
    pub recreate_swapchain : bool
}

impl VulkanoInstance
{
    pub fn new( event_loop : &mut winit::EventsLoop) -> VulkanoInstance
    {
        let vulkano_instance =
        {
            let extensions = vulkano_win_frankenstein::required_extensions();
            Instance::new(None, &extensions, None).expect("Could not create Vulkan instance!")
        };
        
        //move this to window ?
        let window = winit::WindowBuilder::new().build_vk_surface(&event_loop, vulkano_instance.clone()).expect("Could not create window!");
        
        let physical = PhysicalDevice::enumerate(&vulkano_instance).next().expect("Could not get physical device!");
        println!("Using device: {}, (type: {:?})", physical.name(), physical.ty());

        let queue_family = physical.queue_families().find(|&q|
        {
            q.supports_graphics() && window.is_supported(q).unwrap_or(false)
        }).expect("Could not find a queue that supports graphics commands!");

        let(device, mut graphics_queues) = 
        {
            let device_ext = DeviceExtensions
            {
                khr_swapchain: true,
                .. DeviceExtensions::none()
            };

            Device::new(physical, physical.supported_features(), &device_ext, 
            [(queue_family, 0.5)].iter().cloned()).expect("Could not create device!")
        };

        let graphics_queue = graphics_queues.next().expect("Could not get graphics queue from list!");

        let mut dimensions = 
        {   
            let (width, height) = window.window().get_inner_size().expect("Could not get window inner size!");
            [width, height]
        };

        let (swapchain, images) = 
        {
            let surface_capabilities = window.capabilities(physical).expect("Could not get surface capabilities");

            dimensions = surface_capabilities.current_extent.unwrap_or(dimensions);

            let alpha_mode = surface_capabilities.supported_composite_alpha.iter().next().expect("No supported alpha mode for surface!");

            let format = surface_capabilities.supported_formats[0].0;

            Swapchain::new(
                device.clone(), 
                window.clone(), 
                surface_capabilities.min_image_count, 
                format, 
                dimensions, 
                1, 
                surface_capabilities.supported_usage_flags,
                &graphics_queue, 
                SurfaceTransform::Identity, 
                alpha_mode, 
                PresentMode::Mailbox, 
                true, 
                None).expect("Could not create swapchain!")
        };

        let start = Instant::now();
        let previous_frame_time = start;

        let previous_frame_end_future = Box::new(now(device.clone())) as Box<GpuFuture>;

        let vertex_shader = vs::Shader::load(device.clone()).expect("Could not create vertex shader module!");
        let fragment_shader = fs::Shader::load(device.clone()).expect("Could not create fragment shader module!");

        let render_pass = Arc::new(single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).expect("Could not create render pass!"));

        VulkanoInstance{
            device,
            start,
            previous_frame_time,
            previous_frame_end_future,
            window,
            swapchain,
            images,
            render_pass,
            graphics_queue,
            dimensions,
            vertex_shader,
            fragment_shader,
            image_index : 0usize,
            acquire_future : None,
            command_buffer : None,
            recreate_swapchain : false,
       }
    }
}

pub trait PipelineImplementer
{
    fn begin_render(self) -> Self;

    fn end_render(self) -> Self;

    fn recreate_swapchain(&mut self);
}

impl PipelineImplementer for VulkanoInstance
{
    fn recreate_swapchain(&mut self)
    {
        let dimensions = 
        {
            let (width, height) = self.window.window().get_inner_size().expect("Could not get window inner size!");
            [width, height]
        };

        let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimension(dimensions) 
        {
            Ok(r) => r,
            Err(SwapchainCreationError::UnsupportedDimensions) => {
                    return;
                    }
            Err(err) => panic!("{:?}", err)
        };

        mem::replace(&mut self.swapchain, new_swapchain);
        mem::replace(&mut self.images, new_images);

        self.recreate_swapchain = false;
    }

    fn begin_render(mut self) -> Self
    {
        let mut framebuffers : Option<Vec<Arc<Framebuffer<_,_>>>> = None;

        let now = Instant::now();

        // we sleep for 2 seconds
        let time_elapsed = (now.duration_since(self.start).subsec_nanos() as f32) * 0.000000001f32 + now.duration_since(self.start).as_secs() as f32;

        self.previous_frame_time = now;

        let vertex_buffer = 
        {
            CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), [
            Vertex { position: [time_elapsed.cos(), time_elapsed.cos()  * 2f32 + 0.25 ] },
            Vertex { position: [time_elapsed.sin(), 0.5] },
            Vertex { position: [0.25, -0.1] }
        ].iter().cloned()).expect("Could not create vertex buffer!")
        };

        let vertex_buffer_igen = 
        {
            CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), [
            Vertex { position: [time_elapsed.sin(), time_elapsed.sin()  * 2f32 + 0.25 ] },
            Vertex { position: [time_elapsed.cos(), 0.5] },
            Vertex { position: [0.25, -0.1] }
        ].iter().cloned()).expect("Could not create vertex buffer!")
        };

        self.previous_frame_end_future.cleanup_finished();

        if self.recreate_swapchain
        {
            self.recreate_swapchain();
        }

        let (image_index, acquire_future) = match swapchain::acquire_next_image(self.swapchain.clone(), None)
        {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => 
            {
                self.recreate_swapchain = true;
                return self;
            },
            Err(err) => panic!("{:?}", err)
        };

                //this is where we do stuff
        let pipeline = Arc::new(
            GraphicsPipeline::start()
            .vertex_input_single_buffer()
            .vertex_shader(self.vertex_shader.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(self.fragment_shader.main_entry_point(), ())
            .render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
            .build(self.device.clone())
            .unwrap());

        if framebuffers.is_none()
        {
            let new_framebuffers = Some(self.images.iter().map(|image| 
            {
                Arc::new(Framebuffer::start(self.render_pass.clone()).add(image.clone()).unwrap().build().unwrap())
            }).collect::<Vec<_>>());

            mem::replace(&mut framebuffers, new_framebuffers);
        }

        self.acquire_future = Some(acquire_future);
        self.image_index = image_index;

        self.command_buffer = Some(AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.graphics_queue.family()).unwrap()
            .begin_render_pass(framebuffers.as_ref().unwrap()[image_index].clone(), false, 
            vec![[100f32 / 255f32, 149f32 / 255f32, 237f32 / 255f32, 1.0].into()]).unwrap()
            .draw(pipeline.clone(),
            DynamicState{
                line_width: None,
                viewports: Some(vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [self.dimensions[0] as f32, self.dimensions[1] as f32],
                    depth_range: 0.0 .. 1.0,
                }]),
                scissors: None,
            },
            vertex_buffer.clone(), (), ())
            .unwrap()
            .draw(pipeline.clone(),
            DynamicState{
                line_width: None,
                viewports: Some(vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [self.dimensions[0] as f32, self.dimensions[1] as f32],
                    depth_range: 0.0 .. 1.0,
                }]),
                scissors: None,
            },
            vertex_buffer_igen.clone(), (), ())
            .unwrap()
            .end_render_pass()
            .unwrap().build().unwrap());

        self
    }

    fn end_render(mut self) -> Self
    {
        let future = self.previous_frame_end_future.join(self.acquire_future.unwrap());
        let future = future.then_execute(self.graphics_queue.clone(), self.command_buffer.unwrap()).unwrap();
        let future = future.then_swapchain_present(self.graphics_queue.clone(), self.swapchain.clone(), self.image_index);
        let future = future.then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end_future = Box::new(future) as Box<_>;
            }
            Err(FlushError::OutOfDate) => {
                self.previous_frame_end_future = Box::new(now(self.device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end_future = Box::new(now(self.device.clone())) as Box<_>;
            }
        }

        self.command_buffer = None;
        self.acquire_future = None;

        self
    }
}
