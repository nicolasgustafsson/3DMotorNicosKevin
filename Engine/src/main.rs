extern crate winit;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

mod vulkano_win_frankenstein;
mod vulkano_instance;
mod benchmarks;

use benchmarks::render_benchmarks;

fn main() {

    let mut event_loop = winit::EventsLoop::new();

    let mut instance = vulkano_instance::VulkanoInstance::new(&mut event_loop);

    let mut run = true;

    let bench_length = std::time::Duration::new(3, 0);

    let mut benchmarker = benchmarks::benchmarker::Benchmarker::new(vec!(
        Box::new(render_benchmarks::TriangleBenchmark::new(bench_length, 10, 10)),
        Box::new(render_benchmarks::TriangleBenchmark::new(bench_length, 1, 1)),
        Box::new(render_benchmarks::TriangleBenchmark::new(bench_length, 3, 3)),
        ));

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

        benchmarker.tick_tests(&mut instance);
    }
}
