extern crate winit;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

mod vulkano_win_frankenstein;
mod vulkano_instance;

use vulkano_instance::PipelineImplementer;

fn main() {

    let mut event_loop = winit::EventsLoop::new();

    let mut instance = vulkano_instance::VulkanoInstance::new(&mut event_loop);

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

        instance.begin_render();

        if instance.should_recreate_swapchain
        {
            continue;
        }

        instance = instance.end_render();
    }
}

/*

    */