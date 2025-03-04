use glium::backend::glutin::glutin;
use glium::{
    glutin::{
        context::{ContextAttributes, NotCurrentGlContext, PossiblyCurrentGlContext},
        display::GlDisplay,
        surface::WindowSurface,
    },
    Surface,
};
use std::num::NonZero;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle},
    window::{Window as WinitWindow, WindowId},
};

pub struct Window {
    window: Option<WinitWindow>,
    display: Option<glium::backend::glutin::Display<glutin::surface::WindowSurface>>,
}

impl ApplicationHandler for Window {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(WinitWindow::default_attributes())
                .unwrap(),
        );

        let raw_display_handle = self.window.as_ref().unwrap().raw_display_handle().unwrap();
        let raw_window_handle = self.window.as_ref().unwrap().raw_window_handle().unwrap();

        let display = Some(glutin::display::Display::Egl(
            unsafe { glutin::api::egl::display::Display::new(raw_display_handle) }.unwrap(),
        ));

        let config = unsafe {
            &display
                .as_ref()
                .unwrap()
                .find_configs(glutin::config::ConfigTemplate::default())
                .unwrap()
                .nth(0)
                .unwrap()
        };

        let surface = Some(
            unsafe {
                display.as_ref().unwrap().create_window_surface(
                    config,
                    &glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
                        raw_window_handle,
                        NonZero::new(100).unwrap(),
                        NonZero::new(100).unwrap(),
                    ),
                )
            }
            .unwrap(),
        );

        let context = Some(
            unsafe {
                display
                    .as_ref()
                    .unwrap()
                    .create_context(config, &ContextAttributes::default())
                    .unwrap()
                    .make_current(surface.as_ref().unwrap())
            }
            .unwrap(),
        );

        self.display =
            Some(glium::backend::glutin::Display::new(context.unwrap(), surface.unwrap()).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let mut frame = self.display.as_ref().unwrap().draw();
                frame.clear(None, Some((1.0, 0.0, 0.0, 1.0)), true, None, None);
                frame.finish().expect("Failed to finish frame draw");
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

impl Window {
    pub fn new() -> Self {
        Self {
            window: None,
            display: None,
        }
    }

    pub fn open(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}
