extern crate winit;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

mod vulkano_win_frankenstein;

/*use vulkano::instance::Instance;
use vulkano::device::Device;


//use vulkano::swapchain;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
//use vulkano::swapchain::AcquireError;
//use vulkano::swapchain::SwapchainCreationError;

use vulkano::pipeline::GraphicsPipeline;

use vulkano::framebuffer::Subpass;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use std::sync::Arc;*/

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Device;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::instance::Instance;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;

use std::sync::Arc;
use std::mem;


use vulkano_win_frankenstein::vulkano_win_frankenstein::VkSurfaceBuild;

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

    #[derive(Debug, Clone)]
    struct Vertex { position: [f32; 2] }
    impl_vertex!(Vertex, position);

    let vertex_buffer = 
    {
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

    let mut run = true;



    /*WELP START */
        let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_,_>>>> = None;

    // Initialization is finally finished!

    // In some situations, the swapchain will become invalid by itself. This includes for example
    // when the window is resized (as the images of the swapchain will no longer match the
    // window's) or, on Android, when the application went to the background and goes back to the
    // foreground.
    //
    // In this situation, acquiring a swapchain image or presenting it will return an error.
    // Rendering to an image of that swapchain will not produce any error, but may or may not work.
    // To continue rendering, we need to recreate the swapchain by creating a new swapchain.
    // Here, we remember that we need to do this for the next loop iteration.
    let mut recreate_swapchain = false;

    // In the loop below we are going to submit commands to the GPU. Submitting a command produces
    // an object that implements the `GpuFuture` trait, which holds the resources for as long as
    // they are in use by the GPU.
    //
    // Destroying the `GpuFuture` blocks until the GPU is finished executing it. In order to avoid
    // that, we store the submission of the previous frame here.
    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    loop {
        // It is important to call this function from time to time, otherwise resources will keep
        // accumulating and you will eventually reach an out of memory error.
        // Calling this function polls various fences in order to determine what the GPU has
        // already processed, and frees the resources that are no longer needed.
        previous_frame_end.cleanup_finished();

        // If the swapchain needs to be recreated, recreate it
        if recreate_swapchain {
            // Get the new dimensions for the viewport/framebuffers.
            dimensions = {
                let (new_width, new_height) = window.window().get_inner_size().unwrap();
                [new_width, new_height]
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

            mem::replace(&mut swapchain, new_swapchain);
            mem::replace(&mut images, new_images);

            framebuffers = None;

            recreate_swapchain = false;
        }

        // Because framebuffers contains an Arc on the old swapchain, we need to
        // recreate framebuffers as well.
        if framebuffers.is_none() {
            let new_framebuffers = Some(images.iter().map(|image| {
                Arc::new(Framebuffer::start(render_pass.clone())
                         .add(image.clone()).unwrap()
                         .build().unwrap())
            }).collect::<Vec<_>>());
            mem::replace(&mut framebuffers, new_framebuffers);
        }

        // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
        // no image is available (which happens if you submit draw commands too quickly), then the
        // function will block.
        // This operation returns the index of the image that we are allowed to draw upon.
        //
        // This function can block if no image is available. The parameter is an optional timeout
        // after which the function call will return an error.
        let (image_num, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(),
                                                                              None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            },
            Err(err) => panic!("{:?}", err)
        };

        // In order to draw, we have to build a *command buffer*. The command buffer object holds
        // the list of commands that are going to be executed.
        //
        // Building a command buffer is an expensive operation (usually a few hundred
        // microseconds), but it is known to be a hot path in the driver and is expected to be
        // optimized.
        //
        // Note that we have to pass a queue family when we create the command buffer. The command
        // buffer will only be executable on that given queue family.
        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), graphics_queue.family()).unwrap()
            // Before we can draw, we have to *enter a render pass*. There are two methods to do
            // this: `draw_inline` and `draw_secondary`. The latter is a bit more advanced and is
            // not covered here.
            //
            // The third parameter builds the list of values to clear the attachments with. The API
            // is similar to the list of attachments when building the framebuffers, except that
            // only the attachments that use `load: Clear` appear in the list.
            .begin_render_pass(framebuffers.as_ref().unwrap()[image_num].clone(), false,
                               vec![[0.0, 0.0, 1.0, 1.0].into()])
            .unwrap()

            // We are now inside the first subpass of the render pass. We add a draw command.
            //
            // The last two parameters contain the list of resources to pass to the shaders.
            // Since we used an `EmptyPipeline` object, the objects have to be `()`.
            .draw(pipeline.clone(),
                  DynamicState {
                      line_width: None,
                      // TODO: Find a way to do this without having to dynamically allocate a Vec every frame.
                      viewports: Some(vec![Viewport {
                          origin: [0.0, 0.0],
                          dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                          depth_range: 0.0 .. 1.0,
                      }]),
                      scissors: None,
                  },
                  vertex_buffer.clone(), (), ())
            .unwrap()

            // We leave the render pass by calling `draw_end`. Note that if we had multiple
            // subpasses we could have called `next_inline` (or `next_secondary`) to jump to the
            // next subpass.
            .end_render_pass()
            .unwrap()

            // Finish building the command buffer by calling `build`.
            .build().unwrap();

        let future = previous_frame_end.join(acquire_future)
            .then_execute(graphics_queue.clone(), command_buffer).unwrap()

            // The color output is now expected to contain our triangle. But in order to show it on
            // the screen, we have to *present* the image by calling `present`.
            //
            // This function does not actually present the image immediately. Instead it submits a
            // present command at the end of the queue. This means that it will only be presented once
            // the GPU has finished executing the command buffer that draws the triangle.
            .then_swapchain_present(graphics_queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush().unwrap();
        previous_frame_end = Box::new(future) as Box<_>;

        // Note that in more complex programs it is likely that one of `acquire_next_image`,
        // `command_buffer::submit`, or `present` will block for some time. This happens when the
        // GPU's queue is full and the driver has to wait until the GPU finished some work.
        //
        // Unfortunately the Vulkan API doesn't provide any way to not wait or to detect when a
        // wait would happen. Blocking may be the desired behavior, but if you don't want to
        // block you should spawn a separate thread dedicated to submissions.

        // Handling the window events in order to close the program when the user wants to close
        // it.
        let mut done = false;
        //events_loop.poll_events(|ev| {
        //    match ev {
        //        winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } => done = true,
        //        winit::Event::WindowEvent { event: winit::WindowEvent::Resized(_, _), .. } => recreate_swapchain = true,
        //        _ => ()
        //    }
        //});
        //if done { return; 
        }

        //WELP END






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
