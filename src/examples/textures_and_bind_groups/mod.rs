mod texture;

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
    },
    application::ApplicationHandler,
};
use async_std::task;
use image::GenericImageView;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
];
const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

struct Application<'window> {
    window: Option<Arc<Window>>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'window>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    shader: Option<wgpu::ShaderModule>,
    pipeline_layout: Option<wgpu::PipelineLayout>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    buffers: Vec<wgpu::Buffer>,
    diffuse_bind_group: Option<wgpu::BindGroup>,
    diffuse_texture: Option<texture::Texture>,
}

impl<'window> Application<'window> {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(WindowAttributes::default()
            .with_title("My Window")
            .with_inner_size(LogicalSize::new(512, 512))
            .with_position(Position::Physical(PhysicalPosition::new(1800,500))))
            .expect("Failed to create window"));

        let surface = self.instance.create_surface(window.clone()).unwrap();
        let surface_format= surface.get_capabilities(&self.adapter).formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&self.device, &surface_config);

        // SHADER
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // TEXTURE
        let diffuse_bytes = include_bytes!("UV_Grid.png");
        // let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();

        let diffuse_texture = texture::Texture::from_bytes(
            &self.device,
            &self.queue,
            diffuse_bytes,
            "diffuse_texture"
        ).unwrap();

        let texture_bind_group_layout = self.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout")
            }
        );

        let diffuse_bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler)
                    }
                ],
                label: Some("diffuse_bind_group")
            },
        );

        // RENDER PIPELINE
        let pipeline_layout = Some(self.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            })).expect("Failed to create pipeline layout");
        let render_pipeline = self.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &Default::default(),
                        zero_initialize_workgroup_memory: true,
                    },
                    buffers: &[
                        Vertex::desc()
                    ],
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
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    ..wgpu::PrimitiveState::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
        });

        // INIT
        self.window = Some(window);
        self.surface = Some(surface);
        self.shader = Some(shader);
        self.pipeline_layout = Some(pipeline_layout);
        self.render_pipeline = Some(render_pipeline);
        self.surface_config = Some(surface_config);
        self.diffuse_texture = Some(diffuse_texture);
        self.diffuse_bind_group = Some(diffuse_bind_group);
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

                    render_pass
                        .set_pipeline(
                            self.render_pipeline
                                .as_ref()
                                .unwrap());
                    render_pass
                        .set_bind_group(
                            0,
                            &self.diffuse_bind_group,
                            &[]);
                    render_pass
                        .set_vertex_buffer(
                            0,
                            self.buffers[0].slice(..));
                    render_pass
                        .set_index_buffer(
                            self.buffers[1].slice(..),
                            wgpu::IndexFormat::Uint16);
                    render_pass
                        .draw_indexed(
                            0..INDICES.len() as u32,
                            0,
                            0..1);
                    render_pass
                        .draw(
                            0..VERTICES.len() as u32,
                            0..1);
                    drop(render_pass);

                    let command_buffer = encoder.finish();
                    self.queue.submit(Some(command_buffer));
                    frame.present();

                    // window.request_redraw();
                    println!("Redraw Requested");
                }
            }
            WindowEvent::Resized(size) => {
                self.surface_config.as_mut().unwrap().width = size.width;
                self.surface_config.as_mut().unwrap().height = size.height;
                self.surface.as_ref().unwrap().configure(&self.device, &self.surface_config.as_ref().unwrap());
                println!("Window resized: {:?}", size);
            }
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
    // winit
    let event_loop = EventLoop::new().unwrap();

    // wgpu
    let instance= wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default()});
    let adapter= task::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();
    let (device, queue)= task::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None, )).unwrap();

    // buffers
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let mut application = Application{
        window: None,
        surface: None,
        surface_config: None,
        instance,
        adapter,
        device,
        queue,
        shader: None,
        pipeline_layout: None,
        render_pipeline: None,
        buffers: vec![vertex_buffer, index_buffer],
        diffuse_bind_group: None,
        diffuse_texture: None,
    };

    event_loop.run_app(&mut application).expect("Failed to start event_loop");
    println!("base example");
}