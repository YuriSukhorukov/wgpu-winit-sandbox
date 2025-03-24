use std::{
    sync::Arc,
};
use winit::{
    event_loop::{
        EventLoop,
        ActiveEventLoop
    },
    event::{
        DeviceEvent,
        DeviceId,
        StartCause,
        WindowEvent
    },
    window::{
        Window,
        WindowId,
        WindowAttributes
    },
    dpi::{
        LogicalSize,
        Position,
        PhysicalPosition,
        PhysicalSize
    },
    application::ApplicationHandler,
};
use async_std::task;

struct Application<'window> {
    window: Option<Arc<Window>>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'window>>,
    shader: Option<wgpu::ShaderModule>,
    pipeline_layout: Option<wgpu::PipelineLayout>,
    render_pipeline: Option<wgpu::RenderPipeline>,
}

impl<'window> Application<'window> {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(WindowAttributes::default()
            .with_title("My Window")
            .with_inner_size(LogicalSize::new(512, 512))
            .with_position(Position::Physical(PhysicalPosition::new(1800,500)))).expect("Failed to create window"));

        let surface = self.instance.create_surface(window.clone()).unwrap();
        let surface_format= surface.get_capabilities(&self.adapter).formats[0];
        surface.configure(&self.device, &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });

        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = Some(self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        })).expect("Failed to create pipeline layout");

        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &Default::default(),
                    zero_initialize_workgroup_memory: true,
                },
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &Default::default(),
                    zero_initialize_workgroup_memory: true,
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        self.window = Some(window);
        self.surface = Some(surface);
        self.shader = Some(shader);
        self.pipeline_layout = Some(pipeline_layout);
        self.render_pipeline = Some(render_pipeline);
    }
}
impl ApplicationHandler for Application<'_> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        let _ = (event_loop, cause);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() { &self.init(event_loop); }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => { event_loop.exit(); }
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    let frame = self.surface.as_ref().unwrap().get_current_texture().unwrap();
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: Some("Render Encoder")});
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0 }),
                                store: wgpu::StoreOp::Store,
                            }
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
                    render_pass.draw(0..3, 0..1);
                    drop(render_pass);
                    let command_buffer = encoder.finish();
                    self.queue.submit(Some(command_buffer));
                    frame.present();

                    window.request_redraw();
                    println!("Redraw Requested");
                }
            }
            WindowEvent::Resized(size) => { println!("Window resized: {:?}", size); }
            _ => {}
        }
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {}

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {}

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {}
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let instance= wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default()});
    let adapter= task::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();
    let (device, queue)= task::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None, )).unwrap();

    let mut application = Application{
        window: None,
        surface: None,
        instance,
        adapter,
        device,
        queue,
        shader: None,
        pipeline_layout: None,
        render_pipeline: None,
    };

    event_loop.run_app(&mut application).expect("Failed to start event_loop");
    println!("base example");
}