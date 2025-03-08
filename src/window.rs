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
    program,
    texture::{MipmapsOption, RawImage2d, UncompressedFloatFormat},
    uniform,
    uniforms::{ImageUnitFormat, MagnifySamplerFilter, MinifySamplerFilter},
    Blend, DrawParameters, Frame, IndexBuffer, Program, Surface, Texture2d, VertexBuffer,
};
use image::ImageReader;
use nalgebra::{Matrix4, Orthographic3, Vector3, Vector4};
use std::{borrow::Borrow, collections::HashMap, num::NonZero, path::Path};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle},
    window::{Window as WinitWindow, WindowId},
};

#[derive(Copy, Clone)]
struct Vertex {
    a_position: [f32; 3],
    a_tex_coord: [f32; 2],
}
implement_vertex!(Vertex, a_position, a_tex_coord);

struct Rectangle {
    vao: VertexBuffer<Vertex>,
    ebo: IndexBuffer<u16>,
}

impl Rectangle {
    pub fn create(display: &Display<WindowSurface>) -> Self {
        let vertex_data = [
            Vertex {
                a_position: [-1.0, -1.0, 0.0],
                a_tex_coord: [0.0, 0.0],
            },
            Vertex {
                a_position: [-1.0, 1.0, 0.0],
                a_tex_coord: [0.0, 1.0],
            },
            Vertex {
                a_position: [1.0, -1.0, 0.0],
                a_tex_coord: [1.0, 0.0],
            },
            Vertex {
                a_position: [1.0, 1.0, 0.0],
                a_tex_coord: [1.0, 1.0],
            },
        ];
        let index_data = [0, 1, 2, 3, 2, 1];
        Self {
            vao: VertexBuffer::new(display, &vertex_data).unwrap(),
            ebo: IndexBuffer::new(display, PrimitiveType::TrianglesList, &index_data).unwrap(),
        }
    }
    pub fn with_uv(display: &Display<WindowSurface>, custom_uv: [[f32; 2]; 4]) -> Self {
        let vertex_data = [
            Vertex {
                a_position: [-1.0, -1.0, 0.0],
                a_tex_coord: custom_uv[0],
            },
            Vertex {
                a_position: [-1.0, 1.0, 0.0],
                a_tex_coord: custom_uv[1],
            },
            Vertex {
                a_position: [1.0, -1.0, 0.0],
                a_tex_coord: custom_uv[2],
            },
            Vertex {
                a_position: [1.0, 1.0, 0.0],
                a_tex_coord: custom_uv[3],
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
    font_texture: Option<Texture2d>,
    character_rects: HashMap<char, Rectangle>,
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
                    layout (location=0) in vec3 a_position;
                    layout (location=1) in vec2 a_tex_coord;
                    uniform mat4 transform;
                    out vec2 texCoord;
                    void main() {
                        gl_Position = transform * vec4(a_position, 1.0);
                        texCoord = a_tex_coord;
                    }
                "#,
                fragment: r#"
                    #version 330 core
                    out vec4 color;
                    in vec2 texCoord;
                    uniform sampler2D font_texture;
                    void main() {
                        //color = vec4(texCoord.x, texCoord.y, 0, 1);
                        color = texture(font_texture, texCoord);
                    }
                "#
            })
            .unwrap(),
        );

        self.load_font();

        let inner_size = self.window.as_ref().unwrap().inner_size();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let mut frame = self.display.as_ref().unwrap().draw();
                frame.clear(None, Some((0.8, 0.8, 0.8, 1.0)), true, None, None);
                self.render_text("QUICKBROWNFOXJUMPEDOVERTHELAZYDOG", &mut frame);
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
            font_texture: None,
            character_rects: HashMap::new(),
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
        let offset = 50;
        for i in 0..text.len() {
            let coordinates =
                self.screen_to_opengl_coordinates((offset * i + offset / 2) as u32, 500 as u32);

            let size = {
                let size = self.screen_to_relative_coordinates(offset as u32, offset as u32);
                Vector3::new(size[0], size[1], 1.0)
            };

            let mat4 = Matrix4::identity()
                .append_nonuniform_scaling(&size)
                .append_translation(&Vector3::new(coordinates[0], coordinates[1], 0.0));

            let compiled_matrix = TryInto::<[[f32; 4]; 4]>::try_into(mat4.data.0).unwrap();

            let uniforms = uniform![
                transform: compiled_matrix,
                font_texture: self.font_texture
                    .as_ref()
                    .unwrap()
            ];

            frame
                .draw(
                    &self
                        .character_rects
                        .get(&text.chars().nth(i).unwrap())
                        .unwrap()
                        .vao,
                    &self
                        .character_rects
                        .get(&text.chars().nth(i).unwrap())
                        .unwrap()
                        .ebo,
                    self.program.as_ref().unwrap(),
                    &uniforms,
                    &DrawParameters {
                        blend: Blend::alpha_blending(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }

    pub fn load_font(&mut self) {
        let image_data = ImageReader::open("textures/font.png")
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        let raw_image = RawImage2d::from_raw_rgba_reversed(
            &image_data.clone().into_raw(),
            image_data.dimensions(),
        );

        self.font_texture =
            Some(Texture2d::new(self.display.as_ref().unwrap(), raw_image).unwrap());

        let stride_x = 143.0 / image_data.width() as f32;
        let stride_y = 200.0 / image_data.height() as f32;

        // Load the first row of letters
        for x in 0..14 {
            self.character_rects.insert(
                ('A' as u8 + x as u8) as char,
                Rectangle::with_uv(
                    self.display.as_ref().unwrap(),
                    [
                        [stride_x * x as f32, 1.0 - stride_y],
                        [stride_x * x as f32, 1.0],
                        [stride_x * (x as f32 + 1.0), 1.0 - stride_y],
                        [stride_x * (x as f32 + 1.0), 1.0],
                    ],
                ),
            );
        }

        // Load the second row of letters
        for x in 0..12 {
            self.character_rects.insert(
                ('O' as u8 + x as u8) as char,
                Rectangle::with_uv(
                    self.display.as_ref().unwrap(),
                    [
                        [stride_x * x as f32, 1.0 - stride_y * 2.0],
                        [stride_x * x as f32, 1.0 - stride_y],
                        [stride_x * (x as f32 + 1.0), 1.0 - stride_y * 2.0],
                        [stride_x * (x as f32 + 1.0), 1.0 - stride_y],
                    ],
                ),
            );
        }
    }

    pub fn open(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}
