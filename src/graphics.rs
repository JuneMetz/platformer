#[derive(Debug)]
pub struct Graphics {
    window: std::sync::Arc<winit::window::Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl Graphics {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.surface_config.width = new_size.width.max(1);
        self.surface_config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn draw(&mut self) {
        log::info!("attempting to get SurfaceTexture, can panic");
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to aquire next swap chain texture.");

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        log::info!("creating draw command list to submit to queue");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r_pass.set_pipeline(&self.render_pipeline);
            log::info!("actual draw call, note that this draw call assumes that at this point the v&i buffers are set up");
            r_pass.draw(0..3, 0..1);
        } // `r_pass` dropped here
        log::info!("draw commands submitted to queue");
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

fn create_pipeline(device: &wgpu::Device, swap_chain_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    log::info!("loading shader shader.wgsl");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    log::info!("creating render pipeline, vertex position determined by vertex_index");
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(swap_chain_format.into())],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    })
}

pub async fn create_graphics(window: std::sync::Arc<winit::window::Window>, proxy: winit::event_loop::EventLoopProxy<Graphics>) {
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(std::sync::Arc::clone(&window)).unwrap();
    log::info!("requesting adapter (GPU), note that the request is async and can panic");
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(), // Power preference for the device
            force_fallback_adapter: false, // Indicates that only a fallback ("software") adapter can be used
            compatible_surface: Some(&surface), // Guarantee that the adapter can render to this surface
        })
        .await
        .expect("Could not get an adapter (GPU).");
    log::info!("requesting device (GPU), note that the request is async and can panic. Will panic depending on required features");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(), // Specifies the required features by the device request. Fails if the adapter can't provide them.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            },
        )
        .await
        .expect("Failed to get device");

    // Get physical pixel dimensiosn inside the window
    let size = window.inner_size();
    // Make the dimensions at least size 1, otherwise wgpu would panic
    let width = size.width.max(1);
    let height = size.height.max(1);
    let surface_config = surface.get_default_config(&adapter, width, height).unwrap();

    surface.configure(&device, &surface_config);

    let render_pipeline = create_pipeline(&device, surface_config.format);

    let vertex_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: None,
            size: 0,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }
    );

    let gfx = Graphics {
        window: window.clone(),
        instance: instance,
        surface: surface,
        surface_config: surface_config,
        adapter: adapter,
        device: device,
        queue: queue,
        render_pipeline: render_pipeline,
        vertex_buffer: vertex_buffer,
    };
    log::info!("emiting graphics event");
    let _ = proxy.send_event(gfx);
}