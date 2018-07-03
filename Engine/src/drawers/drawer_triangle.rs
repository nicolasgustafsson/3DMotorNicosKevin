use drawers::drawer_base::Drawer;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Device;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;

use std::sync::Arc;
use std::vec::Vec;

#[derive(Debug, Clone)]
struct Vertex { position: [f32; 2] }
impl_vertex!(Vertex, position);

mod vs
{
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
    #version 450
        layout(location = 0) in vec2 position;
        void main() 
        {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "]
    struct _Dummy;
}

mod fs
{
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
    #version 450
        layout(location = 0) out vec4 f_color;
        void main() 
        {
            f_color = vec4(0.0, 0.0, 1.0, 1.0);
        }
    "]
    struct _Dummy;
}

pub struct TriangleDrawer
{
    device : Arc<Device>,
    vertex_shader : vs::Shader,
    fragment_shader : fs::Shader,
    triangle_list : Vec<[[f32; 2]; 3]>
}

impl TriangleDrawer
{
    pub fn new(device : Arc<Device>) -> TriangleDrawer
    {
        let vertex_shader = vs::Shader::load(device.clone()).expect("Could not create vertex shader module for TriangleDrawer!");
        let fragment_shader = fs::Shader::load(device.clone()).expect("Could not create fragment shader module for TriangleDrawer!");

        TriangleDrawer
        {
            device,
            vertex_shader,
            fragment_shader,
            triangle_list : Vec::new()
        }
    }

    pub fn draw_triangle(&mut self, points : [[f32; 2]; 3])
    {
        self.triangle_list.push(points);
    }
}

impl Drawer for TriangleDrawer
{
    fn render(&mut self, dimensions : [u32; 2], command_buffer_builder : AutoCommandBufferBuilder, render_pass : Arc<RenderPassAbstract + Send + Sync>) -> AutoCommandBufferBuilder
    {
        let mut command_buffer_builder = command_buffer_builder;
        for triangle in self.triangle_list.clone()
        {
            let pipeline = Arc::new(
            GraphicsPipeline::start()
            .vertex_input_single_buffer()
            .vertex_shader(self.vertex_shader.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(self.fragment_shader.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(self.device.clone())
            .unwrap());

        let vertex_buffer = 
        {
            CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), [
                Vertex {position: triangle[0]},
                Vertex {position: triangle[1]},
                Vertex {position: triangle[2]}
            ].iter().cloned()).expect("Could not create vertex buffer!")
        };
        
        command_buffer_builder = command_buffer_builder.draw(pipeline.clone(),             
            DynamicState
            {
                line_width: None,
                viewports: Some(vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0 .. 1.0,
                }]),
                scissors: None,
            },             
            vertex_buffer.clone(), (), ()).unwrap();
        }
        self.triangle_list.clear();
        command_buffer_builder
    }
}