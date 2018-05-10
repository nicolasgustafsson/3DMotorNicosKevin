extern crate winit;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

mod vulkano_win_frankenstein;
mod vulkano_instance;

use vulkano_instance::PipelineImplementer;

use vulkano::device::Device;

use vulkano::instance::Instance;

use vulkano::swapchain;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::AcquireError;


use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;

use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;

use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::sync::now;
use vulkano::sync::GpuFuture;

use std::sync::Arc;
use std::mem;

use std::time::Instant;

use vulkano_win_frankenstein::vulkano_win_frankenstein::VkSurfaceBuild;

fn main() {

    let instance = vulkano_instance::VulkanoInstance::new();

    instance.print();

    let vulkano_instance =
    {
        let extensions = vulkano_win_frankenstein::vulkano_win_frankenstein::required_extensions();
        Instance::new(None, &extensions, None).expect("Could not create Vulkan instance!")
    };

    let physical = vulkano::instance::PhysicalDevice::enumerate(&vulkano_instance).next().expect("Could not get physical device!");
    println!("Using device: {}, (type: {:?})", physical.name(), physical.ty());

    let mut event_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new().build_vk_surface(&event_loop, vulkano_instance.clone()).expect("Could not create window!");


    let queue_family = physical.queue_families().find(|&q|
    {
        q.supports_graphics() && window.is_supported(q).unwrap_or(false)
    }).expect("Could not find a queue that supports graphics commands!");

    let (device, mut graphics_queues) = 
    {
        let device_ext = vulkano::device::DeviceExtensions
        {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
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

    let (mut swapchain, mut images) = 
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

        #[derive(Debug, Clone)]
        struct Vertex { position: [f32; 2] }
        impl_vertex!(Vertex, position);

    mod vs {
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[src = "
#version 450
layout(location = 0) in vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
"]
        struct _Dummy;
    }

    mod fs {
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[src = "
#version 450
layout(location = 0) out vec4 f_color;
void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"]
        struct _Dummy;
    }

    let vs = vs::Shader::load(device.clone()).expect("Could not create vertex shader module!");
    let fs = fs::Shader::load(device.clone()).expect("Could not create fragment shader module!");

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

    //this is where we do stuff
    let pipeline = Arc::new(
        GraphicsPipeline::start()
        .vertex_input_single_buffer()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());

    let mut framebuffers : Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_,_>>>> = None;

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    let mut run = true;

    let start = Instant::now();
    let mut previous_frame_time = Instant::now();

    while run
    {
        let now = Instant::now();

        let mut time_elapsed = 0f32;
        // we sleep for 2 seconds
        time_elapsed = (now.duration_since(start).subsec_nanos() as f32) * 0.000000001f32 + now.duration_since(start).as_secs() as f32;

        let nanoseconds : u32 = now.duration_since(previous_frame_time).subsec_nanos();

       // println!("FPS = {}", 1000000000f64 / (nanoseconds as f64));
        
        previous_frame_time = now;

    
        let vertex_buffer = 
        {
                    CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), [
            Vertex { position: [time_elapsed.cos(), time_elapsed.cos()  * 2f32 + 0.25 ] },
            Vertex { position: [time_elapsed.sin(), 0.5] },
            Vertex { position: [0.25, -0.1] }
        ].iter().cloned()).expect("Could not create vertex buffer!")
        };

        previous_frame_end.cleanup_finished();

        if recreate_swapchain
        {
            dimensions = 
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

            recreate_swapchain = false;
        }

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
        });
    }
}