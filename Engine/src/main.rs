extern crate winit;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

mod vulkano_win_frankenstein;
mod vulkano_instance;
mod render_benchmarks;

fn main() {

    let mut event_loop = winit::EventsLoop::new();

    let mut instance = vulkano_instance::VulkanoInstance::new(&mut event_loop);

    let mut run = true;

    let mut test_harness = render_benchmarks::TestHarness::new(200);
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

        test_harness.tick_tests(&mut instance);
    }
}
