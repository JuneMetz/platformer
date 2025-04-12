enum State {
    Ready(crate::graphics::Graphics),
    Init(Option<winit::event_loop::EventLoopProxy<crate::graphics::Graphics>>),
}

pub struct App {
    state: State,
}

impl App {
    pub fn new(event_loop: &winit::event_loop::EventLoop<crate::graphics::Graphics>) -> Self {
        Self {
            state: State::Init(Some(event_loop.create_proxy())),
        }
    }

    fn draw(&mut self) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.draw();
        }
    }

    fn resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.resize(size);
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

                let window = std::sync::Arc::new(
                    event_loop
                        .create_window(win_attr)
                        .expect("create window err."),
                );

                pollster::block_on(crate::graphics::create_graphics(window, proxy));
            }
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, graphics: crate::graphics::Graphics) {
        self.state = State::Ready(graphics);
    }
}