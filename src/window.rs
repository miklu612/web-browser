use glium::backend::glutin::glutin;
use glium::{
    backend::glutin::Display,
    glutin::{
        context::{ContextAttributes, NotCurrentGlContext, PossiblyCurrentGlContext},
        display::GlDisplay,
        surface::WindowSurface,
    },
    implement_vertex,
    index::PrimitiveType,
    program, uniform, Frame, IndexBuffer, Program, Surface, VertexBuffer,
};
use nalgebra::{Matrix4, Orthographic3, Vector3, Vector4};
use std::{borrow::Borrow, num::NonZero};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle},
    window::{Window as WinitWindow, WindowId},
};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}
implement_vertex!(Vertex, position);

struct Rectangle {
    vao: VertexBuffer<Vertex>,
    ebo: IndexBuffer<u16>,
}

impl Rectangle {
    pub fn create(display: &Display<WindowSurface>) -> Self {
        let vertex_data = [
            Vertex {
                position: [-1.0, -1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
            },
        ];
        let index_data = [0, 1, 2, 3, 2, 1];
        Self {
            vao: VertexBuffer::new(display, &vertex_data).unwrap(),
            ebo: IndexBuffer::new(display, PrimitiveType::TrianglesList, &index_data).unwrap(),
        }
    }
}

pub struct Window {
    window: Option<WinitWindow>,
    display: Option<Display<WindowSurface>>,
    rect: Option<Rectangle>,
    program: Option<Program>,
    viewport: Option<Orthographic3<f32>>,
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

        self.rect = Some(Rectangle::create(self.display.as_ref().unwrap()));

        self.program = Some(
            program!(self.display.as_ref().unwrap(),
            330 => {
                vertex: r#"
                    #version 330 core
                    layout (location=0) in vec3 position;
                    uniform mat4 transform;
                    void main() {
                        gl_Position = transform * vec4(position, 1.0);
                    }
                "#,
                fragment: r#"
                    #version 330 core
                    out vec4 color;
                    void main() {
                        color = vec4(1.0, 0.0, 0.0, 1.0);
                    }
                "#
            })
            .unwrap(),
        );

        let inner_size = self.window.as_ref().unwrap().inner_size();
        self.viewport = Some(Orthographic3::new(
            0.0,
            inner_size.width as f32,
            0.0,
            inner_size.height as f32,
            0.1,
            100.0,
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let mut frame = self.display.as_ref().unwrap().draw();
                frame.clear(None, Some((0.2, 0.2, 0.2, 1.0)), true, None, None);
                self.render_text("Hello, World!", &mut frame);
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
            rect: None,
            program: None,
            viewport: None,
        }
    }

    /// Transforms screen coordinates into a -1.0 - 1.0 scale
    pub fn screen_to_opengl_coordinates(&self, x: u32, y: u32) -> [f32; 2] {
        let inner_size = self.window.as_ref().unwrap().inner_size();
        [
            (x as f32 / inner_size.width as f32 - 0.5) * 2.0,
            (y as f32 / inner_size.height as f32 - 0.5) * -2.0,
        ]
    }

    /// Transforms screen coordinates into a 0.0 - 1.0 scale
    pub fn screen_to_relative_coordinates(&self, x: u32, y: u32) -> [f32; 2] {
        let inner_size = self.window.as_ref().unwrap().inner_size();
        [
            x as f32 / inner_size.width as f32,
            y as f32 / inner_size.height as f32,
        ]
    }

    pub fn render_text(&self, text: &str, frame: &mut Frame) {
        for i in 0..text.len() {
            let coordinates = self.screen_to_opengl_coordinates(20 * i as u32, 10 as u32);
            let size = {
                let size = self.screen_to_relative_coordinates(20, 20);
                Vector3::new(size[0], size[1], 1.0)
            };
            let mat4 = Matrix4::identity()
                .append_nonuniform_scaling(&size)
                .append_translation(&Vector3::new(coordinates[0], coordinates[1], 0.0));
            let compiled_matrix = TryInto::<[[f32; 4]; 4]>::try_into(mat4.data.0).unwrap();
            let uniforms = uniform![transform: compiled_matrix];
            frame
                .draw(
                    &self.rect.as_ref().unwrap().vao,
                    &self.rect.as_ref().unwrap().ebo,
                    self.program.as_ref().unwrap(),
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
        }
    }

    pub fn open(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}
