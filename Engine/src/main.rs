extern crate winit;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

use vulkano::instance::Instance;
use vulkano::device::Device;

mod vulkano_win_frankenstein;
use vulkano_win_frankenstein::vulkano_win_frankenstein::VkSurfaceBuild;

//use vulkano::swapchain;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
//use vulkano::swapchain::AcquireError;
//use vulkano::swapchain::SwapchainCreationError;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use std::sync::Arc;

fn main() {
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
            PresentMode::Fifo, 
            true, 
            None).expect("Could not create swapchain!")
    };

    let vertex_buffer = 
    {
        #[derive(Debug, Clone)]
        struct Vertex { position: [f32; 2] }
        impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), [
            Vertex { position: [-0.5, -0.25] },
            Vertex { position: [0.0, 0.5] },
            Vertex { position: [0.25, -0.1] }
        ].iter()).expect("Could not create vertex buffer!");
    };

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
        struct Dummy;
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
        struct Dummy;
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

    let mut run = true;

    while run
    {
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
