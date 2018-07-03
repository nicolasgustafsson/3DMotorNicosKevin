use vulkano::framebuffer::RenderPassAbstract;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use std::sync::Arc;

pub trait Drawer
{
    fn render(&mut self, dimensions : [u32; 2], command_buffer_builder : AutoCommandBufferBuilder, render_pass : Arc<RenderPassAbstract + Send + Sync>) -> AutoCommandBufferBuilder;
}