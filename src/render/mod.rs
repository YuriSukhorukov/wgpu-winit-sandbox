pub mod vertex;

use std::sync::Arc;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
pub use vertex::Vertex;

pub struct Render<'window> {
    window: Option<Arc<winit::window::Window>>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'window>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    shader: Option<wgpu::ShaderModule>,
    pipeline: Option<wgpu::RenderPipeline>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
}

impl<'window> Render<'window> {
    fn init() {}
}
impl winit::application::ApplicationHandler for Render<'_> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        todo!()
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        todo!()
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        todo!()
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        todo!()
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }
}