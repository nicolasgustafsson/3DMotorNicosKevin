#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_window_glutin;
extern crate glutin;

use gfx_core::{Adapter, CommandQueue, Device, Surface, SwapchainExt, FrameSync, GraphicsPoolExt};
use gfx_core::traits::Swapchain;
use glutin::os::windows::{WindowExt};
pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const TRIANGLE: [Vertex; 3] = [
    Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
    Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
    Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] }
];

fn main() {
    //let sdl2_context = sdl2::init().expect("Could not initialize sdl2!");
    //let video_context = sdl2_context.video().expect("Could not get video context from sdl2 context!");
//
    //let mut window = match video_context.window("Awesome engine is awesome (kallehobbe)", 1280, 720).position_centered().opengl().build()
    //{
    //    Ok(window)  => window,
    //    Err(err)    => panic!("Could not create window! {}", err)
    //};

    //println!("{}", video_context.current_video_driver());
    //window.show();

    //update window

    //let mut events = sdl2_context.event_pump().expect("Could not get event pump from sdl2 context!");
//
    //'outer : loop {
    //    for event in events.poll_iter() {
    //        match event {
    //            Event::Quit{..} => break 'outer,
    //            _               => continue
    //        }
    //    }

        //do engine loop thingamajig

    //}
}
