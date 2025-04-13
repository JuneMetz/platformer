enum State {
    Ready(crate::graphics::Graphics),
    Init(Option<winit::event_loop::EventLoopProxy<crate::graphics::Graphics>>),
}

pub struct App {
    state: State,
}

impl App {
    pub fn new(event_loop: &winit::event_loop::EventLoop<crate::graphics::Graphics>) -> Self {
        log::info!("attempting creation of proxy to some effect");
        Self {
            state: State::Init(Some(event_loop.create_proxy())),
        }
    }

    fn draw(&mut self) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.draw();
        } else {
            log::warn!("failed draw call as we are still initializing");
        }
    }

    fn resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.resize(size);
        } else {
            log::warn!("failed resize call as we are still initializing");
        }
    }
}

impl winit::application::ApplicationHandler<crate::graphics::Graphics> for App {
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::Resized(size) => self.resized(size),
            winit::event::WindowEvent::RedrawRequested => self.draw(),
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let State::Init(proxy) = &mut self.state {
            if let Some(proxy) = proxy.take() {
                let mut win_attr = winit::window::Window::default_attributes();

                win_attr = win_attr.with_title("WebGPU example");

                log::info!("creating window creation process, panics on create_window error");
                let window = std::sync::Arc::new(
                    event_loop
                        .create_window(win_attr)
                        .expect("create window err."),
                );
                log::info!("creating graphics object, sending it to proxy to create window");
                pollster::block_on(crate::graphics::create_graphics(window, proxy));
            } else {
                log::warn!("failed to expand EventLoop proxy to concrete type without error");
            }
        } else {
            log::warn!("failed to move EventLoop proxy from inner to outer scope");
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, graphics: crate::graphics::Graphics) {
        self.state = State::Ready(graphics);
    }
}