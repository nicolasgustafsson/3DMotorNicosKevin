extern crate winit;

use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::framebuffer::Framebuffer;
use vulkano::swapchain::Surface;

use vulkano::sync::now;
use vulkano::sync::GpuFuture;

use vulkano_win_frankenstein::vulkano_win_frankenstein;
use vulkano_win_frankenstein::vulkano_win_frankenstein::VkSurfaceBuild;

use std::sync::Arc;
use std::boxed::Box;
use std::time::Instant;
use std::mem;
use std::vec::Vec;

pub struct VulkanoInstance
{
    device : Arc<Device>,
    start : Instant,
    previous_frame_time : Instant,
    previous_frame_end_future : Box<GpuFuture>,
    recreate_swapchain : bool,
    framebuffers : std::option::Option<Vec<Arc<Framebuffer<Arc<RenderPass<CustomRenderPassDesc>>, ((),Arc<SwapchainImage<Window>>)>>>>
 //eEWWWWWWWWWWWWWWWW
}

impl VulkanoInstance
{
    pub fn new() -> VulkanoInstance
    {
        let vulkano_instance =
        {
            let extensions = vulkano_win_frankenstein::required_extensions();
            Instance::new(None, &extensions, None).expect("Could not create Vulkan instance!")
        };

        let mut event_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new().build_vk_surface(&event_loop, vulkano_instance.clone()).expect("Could not create window!");
        
        let physical = PhysicalDevice::enumerate(&vulkano_instance).next().expect("Could not get physical device!");
        println!("Using device: {}, (type: {:?})", physical.name(), physical.ty());

        let queue_family = physical.queue_families().find(|&q|
        {
            q.supports_graphics() && window.is_supported(q).unwrap_or(false)
        }).expect("Could not find a queue that supports graphics commands!");

        let(device, graphics_queue) = 
        {
            let device_ext = DeviceExtensions
            {
                khr_swapchain: true,
                .. DeviceExtensions::none()
            };

            Device::new(physical, physical.supported_features(), &device_ext, 
            [(queue_family, 0.5)].iter().cloned()).expect("Could not create device!")
        };

        let start = Instant::now();
        let previous_frame_time = start;

        let previous_frame_end_future = Box::new(now(device.clone())) as Box<GpuFuture>;

        VulkanoInstance{
            device,
            start,
            previous_frame_time,
            previous_frame_end_future,
            recreate_swapchain : false
        }
    }
}

pub trait PipelineImplementer
{
    fn setup(&self);

    fn begin_render(&mut self);

    fn end_render(&self);

    fn print(&self);
}

impl PipelineImplementer for VulkanoInstance
{
    fn setup(&self)
    {
        
    }
    
    fn begin_render(&mut self)
    {
        let mut framebuffers : Option<Vec<Arc<Framebuffer<_,_>>>> = None;

        let now = Instant::now();

        let mut time_elapsed = 0f32;
        // we sleep for 2 seconds
        time_elapsed = (now.duration_since(self.start).subsec_nanos() as f32) * 0.000000001f32 + now.duration_since(self.start).as_secs() as f32;

        let nanoseconds : u32 = now.duration_since(self.previous_frame_time).subsec_nanos();

       // println!("FPS = {}", 1000000000f64 / (nanoseconds as f64));
        
        self.previous_frame_time = now;

        #[derive(Debug, Clone)]
        struct Vertex { position: [f32; 2] }
        impl_vertex!(Vertex, position);

        let vertex_buffer = 
        {
            CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), [
            Vertex { position: [time_elapsed.cos(), time_elapsed.cos()  * 2f32 + 0.25 ] },
            Vertex { position: [time_elapsed.sin(), 0.5] },
            Vertex { position: [0.25, -0.1] }
        ].iter().cloned()).expect("Could not create vertex buffer!")
        };

        self.previous_frame_end_future.cleanup_finished();

        if self.recreate_swapchain
        {
            let dimensions = 
            {
                let (width, height) = window.window().get_inner_size().expect("Could not get window inner size!");
                [width, height]
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) 
            {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => 
                {
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

            mem::replace(&mut swapchain, new_swapchain);
            mem::replace(&mut images, new_images);

            framebuffers = None;

            self.recreate_swapchain = false;
        }
/*
        if framebuffers.is_none()
        {
            let new_framebuffers = Some(images.iter().map(|image| 
            {
                Arc::new(Framebuffer::start(render_pass.clone()).add(image.clone()).unwrap().build().unwrap())
            }).collect::<Vec<_>>());

            mem::replace(&mut framebuffers, new_framebuffers);
        }

        let (image_index, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None)
        {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => 
            {
                recreate_swapchain = true;
                continue;
            },
            Err(err) => panic!("{:?}", err)
        };

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), graphics_queue.family()).unwrap()
            .begin_render_pass(framebuffers.as_ref().unwrap()[image_index].clone(), false, 
            vec![[100f32 / 255f32, 149f32 / 255f32, 237f32 / 255f32, 1.0].into()]).unwrap()
            .draw(pipeline.clone(),
            DynamicState{
                line_width: None,
                viewports: Some(vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0 .. 1.0,
                }]),
                scissors: None,
            },
            vertex_buffer.clone(), (), ())
            .unwrap()
            .end_render_pass()
            .unwrap().build().unwrap();

        let future = previous_frame_end.join(acquire_future)
        .then_execute(graphics_queue.clone(), command_buffer).unwrap()
        .then_swapchain_present(graphics_queue.clone(), swapchain.clone(), image_index)
        .then_signal_fence_and_flush().unwrap();

        previous_frame_end = Box::new(future) as Box<_>;

        event_loop.poll_events(|event| 
        {
            match event 
            {
                winit::Event::WindowEvent {
                event: winit::WindowEvent::CloseRequested,
                ..
                } => run = false,
                _ => run = true,
            };
        });*/
    }

    fn end_render(&self)
    {
        
    }

    fn print(&self)
    {
        println!("HEJ NICOS");
    }
}
