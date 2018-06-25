extern crate winit;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

mod vulkano_win_frankenstein;
mod vulkano_instance;

use vulkano_instance::PipelineImplementer;
use std::time::Instant;

fn main() {

    let mut event_loop = winit::EventsLoop::new();

    let mut instance = vulkano_instance::VulkanoInstance::new(&mut event_loop);

    let mut run = true;

    let start = Instant::now();
    let mut previous_time = start;

    while run
    {        
        let now = Instant::now();
        let time_elapsed = (now.duration_since(start).subsec_nanos() as f32) * 0.000000001f32 + now.duration_since(start).as_secs() as f32;

        let frame_delta = now - previous_time;

        let frame_delta_ms = frame_delta.subsec_nanos() as f32 * 0.000001f32 + frame_delta.as_secs() as f32 * 1000f32;

        println!("frametime: {} ms", frame_delta_ms);

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

        if instance.should_recreate_swapchain //todo: remove this, get result from begin_render instead
        {
            continue;
        }

        for i in 0..=10
        {
            instance.draw_triangle([
                [time_elapsed.sin() + (i as f32 / 10f32).sin(), time_elapsed.sin()  * 2f32 + 0.25 ], 
                [time_elapsed.cos(), 0.5], 
                [0.25, -0.1]]);

            instance.draw_triangle([
                [time_elapsed.cos() + (i as f32 / 10f32).cos(), time_elapsed.cos()  * 2f32 + 0.25 ], 
                [time_elapsed.sin(), 0.5], 
                [0.25, -0.1]]);
        }

        instance.end_render();
        previous_time = now;
    }
}
