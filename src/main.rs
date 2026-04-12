//! The World — Raw WGPU Triangle
//!
//! Renders a single flat triangle using raw WGPU 0.27 inside Bevy.
//! Uses Bevy only for window creation and event loop.
//! All GPU work is pure WGPU.

use bevy::prelude::*;
use wgpu::util::DeviceExt;

// Prevent Bevy's built-in renderer from interfering
#[derive(Resource)]
struct RendererControl {
    gpu_initialized: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// WGSL SHADERS
// ─────────────────────────────────────────────────────────────────────────────

const VERTEX_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(
    @location(0) color: vec3<f32>,
) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// VERTEX DATA
// ─────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [ 0.0,  0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GPU STATE
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    surface: wgpu::Surface<'static>,
    initialized: bool,
}

impl GpuState {
    async fn new(window: &bevy::window::Window, raw_handle: &bevy::window::RawHandleWrapper) -> Self {
        let size = window.physical_size();
        let size = (size.x.max(1), size.y.max(1));

        // ── Instance ────────────────────────────────────────────────────────────
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // ── Surface from raw window handle ─────────────────────────────────────
        // SAFETY: get_handle returns a ThreadLockedRawWindowHandleWrapper which
        // implements HasWindowHandle + HasDisplayHandle, required by wgpu surface creation.
        let handle_wrapper = unsafe { raw_handle.get_handle() };
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&handle_wrapper).expect("Failed to create surface"))
        }
        .expect("Failed to create surface");

        // ── Adapter ────────────────────────────────────────────────────────────
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to request GPU adapter");

        // ── Device + Queue ───────────────────────────────────────────────────
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("the_world_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to request GPU device");

        // ── Surface Configuration ─────────────────────────────────────────────
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.0,
            height: size.1,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // ── Shader Module ──────────────────────────────────────────────────────
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle_shader"),
            source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
        });

        // ── Pipeline Layout ─────────────────────────────────────────────────────
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // ── Render Pipeline ───────────────────────────────────────────────────
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        // ── Vertex Buffer ─────────────────────────────────────────────────────
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("triangle_vertex_buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            surface,
            initialized: true,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == self.config.width && height == self.config.height {
            return;
        }
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn render_frame(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(o) => o,
            Err(e) => {
                eprintln!("get_current_texture error: {:?}", e);
                return;
            }
        };
        let view = output.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("triangle_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.15, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BEVY APP
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, (init_gpu, render).chain())
        .run();
}

fn setup(world: &mut World) {
    // Insert a marker that we'll use to initialize GPU on first frame
    world.insert_resource(RendererControl { gpu_initialized: false });
}

fn init_gpu(
    mut renderer_control: ResMut<RendererControl>,
    mut commands: Commands,
    world: &World,
) {
    if renderer_control.gpu_initialized {
        return; // Already initialized
    }

    let window_entity = world
        .query_filtered::<Entity, With<bevy::window::PrimaryWindow>>()
        .iter(world)
        .next();

    let Some(window_entity) = window_entity else {
        return; // Window not ready yet
    };

    let window = match world.get::<bevy::window::Window>(window_entity) {
        Some(w) => w,
        None => return,
    };

    let raw_handle = match world.get::<bevy::window::RawHandleWrapper>(window_entity) {
        Some(rh) => rh,
        None => return,
    };

    // Initialize GPU synchronously on first frame
    let gpu = pollster::block_on(GpuState::new(window, raw_handle));
    commands.insert_resource(gpu);
    renderer_control.gpu_initialized = true;
}

fn render(
    mut gpu: ResMut<GpuState>,
    windows: Query<&bevy::window::Window, With<bevy::window::PrimaryWindow>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    // Only resize if really needed - avoid reconfiguring while rendering
    let size = window.physical_size();
    if size.x > 0 && size.y > 0 {
        gpu.resize(size.x, size.y);
    }
    gpu.render_frame();
}
