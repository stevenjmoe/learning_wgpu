use simple_logger::SimpleLogger;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Fullscreen, Window, WindowBuilder},
};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_colour: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline2: wgpu::RenderPipeline,
    use_colour: bool,
    window: Window,
}

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let clear_colour = wgpu::Color::BLACK;
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let shader2 = device.create_shader_module(wgpu::include_wgsl!("challenge_shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let render_pipeline2 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipline 2"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader2,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader2,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let use_colour = false;

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            clear_colour,
            render_pipeline,
            render_pipeline2,
            use_colour,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            println!("{:?}", new_size);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_colour = wgpu::Color {
                    r: position.x,
                    g: position.y,
                    a: 1.0,
                    b: 1.0,
                };
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        logical_key: Key::Named(NamedKey::Space),
                        ..
                    },
                ..
            } => {
                self.use_colour = *state == ElementState::Released;
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_colour),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(if self.use_colour {
                &self.render_pipeline
            } else {
                &self.render_pipeline2
            });

            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() -> Result<(), impl std::error::Error> {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new().unwrap();

    let mut decorations = true;
    let mut minimized = false;
    let mut maximized = false;
    let mut with_min_size = false;
    let mut with_max_size = false;

    let window = WindowBuilder::new()
        .with_title("Window!")
        .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
        .build(&event_loop)
        .unwrap();

    let mut monitor = event_loop
        .available_monitors()
        .next()
        .expect("No monitor found!");
    println!("Monitor: {:?}", monitor.name());

    let mut mode_index = 0;
    let mut mode = monitor
        .video_modes()
        .next()
        .expect("No fullscreen mode found");
    println!("Mode: {mode}");

    let mut state = State::new(window).await;

    event_loop.run(move |event, elwt| {
        println!("{event:?}");

        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),

                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    logical_key: key,
                                    ..
                                },
                            ..
                        } => match key {
                            Key::Named(NamedKey::Escape) => elwt.exit(),
                            Key::Character(ch) => match ch.to_lowercase().as_str() {
                                "f" | "b" if state.window.fullscreen().is_some() => {
                                    state.window.set_fullscreen(None);
                                }
                                "f" => {
                                    let fullscreen = Some(Fullscreen::Exclusive(mode.clone()));
                                    println!("Setting mode: {fullscreen:?}");
                                    state.window.set_fullscreen(fullscreen);
                                }
                                "b" => {
                                    let fullscreen =
                                        Some(Fullscreen::Borderless(Some(monitor.clone())));
                                    println!("Setting mode: {fullscreen:?}");
                                    state.window.set_fullscreen(fullscreen);
                                }
                                "m" => {
                                    mode_index += 1;
                                    if let Some(m) = monitor.video_modes().nth(mode_index) {
                                        mode = m;
                                    } else {
                                        mode_index = 0;
                                        mode = monitor
                                            .video_modes()
                                            .next()
                                            .expect("No fullscreen mode found");
                                    }
                                    println!("Mode: {mode}");
                                }
                                "d" => {
                                    decorations = !decorations;
                                    state.window.set_decorations(decorations);
                                }
                                "x" => {
                                    maximized = !maximized;
                                    state.window.set_maximized(maximized);
                                }
                                "z" => {
                                    minimized = !minimized;
                                    state.window.set_minimized(minimized);
                                }
                                "i" => {
                                    with_min_size = !with_min_size;
                                    let min_size = if with_min_size {
                                        Some(PhysicalSize::new(100, 100))
                                    } else {
                                        None
                                    };

                                    state.window.set_min_inner_size(min_size);
                                    eprintln!(
                                        "Min: {with_min_size}: {min_size:?} => {:?}",
                                        state.window.inner_size()
                                    );
                                }
                                "a" => {
                                    with_max_size = !with_max_size;
                                    let max_size = if with_max_size {
                                        Some(PhysicalSize::new(200, 200))
                                    } else {
                                        None
                                    };

                                    state.window.set_max_inner_size(max_size);
                                    eprintln!(
                                        "Max: {with_max_size}: {max_size:?} => {:?}",
                                        state.window.inner_size()
                                    );
                                }
                                _ => (),
                            },
                            _ => (),
                        },

                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                            state.window.pre_present_notify();
                        }

                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }

                        _ => (),
                    }
                }
            }

            Event::AboutToWait => {
                state.window.request_redraw();
            }

            _ => (),
        }
    })
}
