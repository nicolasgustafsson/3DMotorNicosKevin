extern crate winit;

use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::swapchain;
use vulkano::swapchain::Surface;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::AcquireError;
use vulkano::image::swapchain::SwapchainImage;

use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::device::Queue; 
use winit::Window;

use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::sync::FlushError;

use vulkano_win_frankenstein::vulkano_win_frankenstein;
use vulkano_win_frankenstein::vulkano_win_frankenstein::VkSurfaceBuild;

use std::sync::Arc;
use std::boxed::Box;
use std::mem;
use std::vec::Vec;
use std::option::Option;

use drawers::drawer_base::Drawer;
use drawers::drawer_triangle;

pub enum RenderError
{
    SwapchainOutOfDate
}

pub struct VulkanoInstance
{
    device : Arc<Device>,
    previous_frame_end_future : Option<Box<GpuFuture>>,
    window : Arc<Surface<Window>>,
    swapchain : Arc<Swapchain<Window>>,
    images : Vec<Arc<SwapchainImage<Window>>>,
    render_pass : Arc<RenderPassAbstract + Send + Sync>,
    graphics_queue : Arc<Queue>,
    dimensions : [u32; 2],
    image_index : usize,
    acquire_future : Option<SwapchainAcquireFuture<Window>>,
    command_buffer_builder : Option<AutoCommandBufferBuilder>, //maybe not option?
    triangle_drawer : drawer_triangle::TriangleDrawer,
    pub should_recreate_swapchain : bool
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
        let previous_frame_end_future = Box::new(now(device.clone())) as Box<GpuFuture>;

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

        let triangle_drawer = drawer_triangle::TriangleDrawer::new(device.clone());

        VulkanoInstance{
            device,
            previous_frame_end_future : Some(previous_frame_end_future),
            window,
            swapchain,
            images,
            render_pass,
            graphics_queue,
            dimensions,
            image_index : 0usize,
            acquire_future : None,
            should_recreate_swapchain : false,
            triangle_drawer,
            command_buffer_builder : None,
       }
    }
}

pub trait PipelineImplementer
{
    fn recreate_swapchain(&mut self);

    fn begin_render(&mut self) -> Result<(), RenderError>;

    fn end_render(&mut self);

    fn draw_triangle(&mut self, points : [[f32; 2]; 3]);
}

impl PipelineImplementer for VulkanoInstance
{
    fn recreate_swapchain(&mut self)
    {
        self.dimensions = 
        {
            let (width, height) = self.window.window().get_inner_size().expect("Could not get window inner size!");
            [width, height]
        };

        if self.dimensions[0] <= 0 || self.dimensions[1] <= 0
        {
            return;
        }

        let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimension(self.dimensions) 
        {
            Ok(r) => r,
            Err(SwapchainCreationError::UnsupportedDimensions) => {
                return;
            }
            Err(err) => panic!("{:?}", err)
        };

        mem::replace(&mut self.swapchain, new_swapchain);
        mem::replace(&mut self.images, new_images);

        self.should_recreate_swapchain = false;
    }

    fn begin_render(&mut self) -> Result<(), RenderError>
    {
        let mut framebuffers : Option<Vec<Arc<Framebuffer<_,_>>>> = None;

        self.previous_frame_end_future.as_mut().unwrap().cleanup_finished();

        if self.should_recreate_swapchain
        {
            self.recreate_swapchain();
        }

        let (image_index, acquire_future) = match swapchain::acquire_next_image(self.swapchain.clone(), None)
        {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => 
            {
                self.should_recreate_swapchain = true;
                return Err(RenderError::SwapchainOutOfDate);
            },
            Err(err) => panic!("{:?}", err)
        };

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

        self.command_buffer_builder = Some(AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.graphics_queue.family()).unwrap()
            .begin_render_pass(framebuffers.as_ref().unwrap()[image_index].clone(), false, 
            vec![[100f32 / 255f32, 149f32 / 255f32, 237f32 / 255f32, 1.0].into()]).unwrap());

        Ok(())
    }

    fn end_render(&mut self)
    {
        self.command_buffer_builder = Some(self.triangle_drawer.render(self.dimensions, self.command_buffer_builder.take().unwrap(), self.render_pass.clone()));

        let mut command_buffer_builder = self.command_buffer_builder.take();
        command_buffer_builder = Some(command_buffer_builder.unwrap().end_render_pass().unwrap());

        let command_buffer = command_buffer_builder.unwrap().build().unwrap();

        let acquire = self.acquire_future.take();

        let future  = self.previous_frame_end_future.take().unwrap().join(acquire.unwrap());
        let future  = future.then_execute(self.graphics_queue.clone(), command_buffer).unwrap();
        let future  = future.then_swapchain_present(self.graphics_queue.clone(), self.swapchain.clone(), self.image_index);
        let future  = future.then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end_future = Some(Box::new(future) as Box<_>);
            }
            Err(FlushError::OutOfDate) => {
                self.previous_frame_end_future = Some(Box::new(now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end_future = Some(Box::new(now(self.device.clone())) as Box<_>);
            }
        }
    }

    fn draw_triangle(&mut self, points : [[f32; 2]; 3])
    {
        self.triangle_drawer.draw_triangle(points);
    }
}
