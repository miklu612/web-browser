use crate::html::{Element, Tag};
use crate::render_layout::{ElementRect, Position, Size, Word};
use glium::backend::glutin::glutin;
use glium::{
    backend::glutin::Display,
    glutin::{
        context::{ContextAttributes, NotCurrentGlContext},
        display::GlDisplay,
        surface::WindowSurface,
    },
    implement_vertex,
    index::PrimitiveType,
    program,
    texture::RawImage2d,
    uniform, Blend, DrawParameters, Frame, IndexBuffer, Program, Surface, Texture2d, VertexBuffer,
};
use image::ImageReader;
use nalgebra::{Matrix4, Vector3};
use std::{collections::HashMap, num::NonZero};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::Key,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window as WinitWindow, WindowId},
};

pub enum Color {
    Black,
    LinkBlue,
}

impl Color {
    pub fn to_color(&self) -> [f32; 3] {
        match self {
            Self::Black => [0.0, 0.0, 0.0],
            Self::LinkBlue => [0.3, 0.3, 0.9],
        }
    }
}

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
    elements: Vec<Element>,
    scroll_y: i32,
}

impl ApplicationHandler for Window {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(WinitWindow::default_attributes())
                .unwrap(),
        );

        let raw_display_handle = self
            .window
            .as_ref()
            .unwrap()
            .display_handle()
            .unwrap()
            .as_raw();
        let raw_window_handle = self
            .window
            .as_ref()
            .unwrap()
            .window_handle()
            .unwrap()
            .as_raw();

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

        let surface = unsafe {
            display.as_ref().unwrap().create_window_surface(
                config,
                &glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
                    raw_window_handle,
                    NonZero::new(100).unwrap(),
                    NonZero::new(100).unwrap(),
                ),
            )
        }
        .unwrap();

        let context = unsafe {
            display
                .as_ref()
                .unwrap()
                .create_context(config, &ContextAttributes::default())
                .unwrap()
                .make_current(&surface)
        }
        .unwrap();

        self.display = Some(glium::backend::glutin::Display::new(context, surface).unwrap());

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
                    uniform vec3 color_addition;
                    void main() {
                        color = texture(font_texture, texCoord);
                        color.x = color.x + color_addition.x;
                        color.y = color.y + color_addition.y;
                        color.z = color.z + color_addition.z;
                    }
                "#
            })
            .unwrap(),
        );

        self.load_font();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let mut frame = self.display.as_ref().unwrap().draw();
                frame.clear(None, Some((0.8, 0.8, 0.8, 1.0)), true, None, None);
                self.render_current_page(&mut frame);
                frame.finish().expect("Failed to finish frame draw");
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key.as_ref() {
                Key::Character("j") => self.scroll_y += 10,
                Key::Character("k") => self.scroll_y -= 10,
                _ => (),
            },

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
            elements: Vec::new(),
            scroll_y: 0,
        }
    }

    const FONT_SIZE: f32 = 30.0;

    /// Transforms screen coordinates into a -1.0 - 1.0 scale
    pub fn screen_to_opengl_coordinates(&self, x: i32, y: i32) -> [f32; 2] {
        let inner_size = self.window.as_ref().unwrap().inner_size();
        [
            (x as f32 / inner_size.width as f32 - 0.5) * 2.0,
            (y as f32 / inner_size.height as f32 - 0.5) * -2.0,
        ]
    }

    /// Transforms screen coordinates into a 0.0 - 1.0 scale
    pub fn screen_to_relative_coordinates(&self, x: i32, y: i32) -> [f32; 2] {
        let inner_size = self.window.as_ref().unwrap().inner_size();
        [
            x as f32 / inner_size.width as f32,
            y as f32 / inner_size.height as f32,
        ]
    }

    pub fn render(&mut self, elements: Vec<Element>) {
        self.elements = elements;
        self.open();
    }

    pub fn render_words(
        &self,
        words: &Vec<Word>,
        frame: &mut Frame,
        position: Position,
        font_size: Size,
    ) {
        for word in words {
            let mut character_position = Position::new(word.position.x, word.position.y);
            let scaling = font_size.width as f32 / Self::FONT_SIZE;
            for character in word.word.chars() {
                let gl_position =
                    self.screen_to_opengl_coordinates(character_position.x, character_position.y);
                self.render_character(character, frame, gl_position[0], gl_position[1], scaling);
                character_position.x += font_size.width;
            }
        }
    }

    pub fn render_element_rect_with_offset(
        &self,
        element_rect: &ElementRect,
        frame: &mut Frame,
        position: Position,
    ) {
        if let Some(words) = element_rect.words.as_ref() {
            self.render_words(words, frame, position, element_rect.font_size);
        }
        for child in &element_rect.children {
            let new_position =
                Position::new(position.x + child.position.x, position.y + child.position.y);
            self.render_element_rect_with_offset(&child, frame, new_position);
        }
    }

    pub fn render_element_rect(&self, element_rect: &ElementRect, frame: &mut Frame) {
        self.render_element_rect_with_offset(element_rect, frame, Position::new(0, 0));
    }

    pub fn render_current_page(&self, frame: &mut Frame) {
        let page_layout = self.create_page_layout();
        self.render_element_rect(&page_layout, frame);
    }

    pub fn create_page_layout(&self) -> ElementRect {
        let mut body = None;
        for element in &self.elements[0].children {
            if element.element_type == Tag::Body {
                body = Some(element);
                break;
            }
        }
        let body = body.unwrap();
        let inner_size = self.window.as_ref().unwrap().inner_size();
        ElementRect::from_element(
            body,
            Position::new(20, 20),
            Size::new(inner_size.width as i32 - 20, 9999),
            Size::new(Self::FONT_SIZE as i32, Self::FONT_SIZE as i32),
        )
    }

    pub fn render_character(&self, character: char, frame: &mut Frame, x: f32, y: f32, scale: f32) {
        // If the character is not visibile, don't render it.
        if y < -1.0 || y > 1.0 {
            return;
        }

        let size = self.screen_to_relative_coordinates(
            (Self::FONT_SIZE * scale) as i32,
            (Self::FONT_SIZE * scale) as i32,
        );
        let mat4 = Matrix4::identity()
            .append_nonuniform_scaling(&Vector3::new(size[0], size[1], 1.0))
            .append_translation(&Vector3::new(x, y, 0.0));
        let compiled_matrix = TryInto::<[[f32; 4]; 4]>::try_into(mat4.data.0).unwrap();

        let character_rect = self
            .character_rects
            .get(&character)
            .unwrap_or(self.character_rects.get(&'A').unwrap());

        let uniforms = uniform![
            transform: compiled_matrix,
            font_texture: self.font_texture.as_ref().unwrap(),
            color_addition: Color::Black.to_color()
        ];

        frame
            .draw(
                &character_rect.vao,
                &character_rect.ebo,
                self.program.as_ref().unwrap(),
                &uniforms,
                &DrawParameters {
                    blend: Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
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
                (b'A' + x as u8) as char,
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
                (b'O' + x as u8) as char,
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

        // Load the numbers
        for x in 12..14 {
            self.character_rects.insert(
                (b'0' + (x - 12) as u8) as char,
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

        for x in 0..8 {
            self.character_rects.insert(
                (b'2' + x as u8) as char,
                Rectangle::with_uv(
                    self.display.as_ref().unwrap(),
                    [
                        [stride_x * x as f32, 1.0 - stride_y * 3.0],
                        [stride_x * x as f32, 1.0 - stride_y * 2.0],
                        [stride_x * (x as f32 + 1.0), 1.0 - stride_y * 3.0],
                        [stride_x * (x as f32 + 1.0), 1.0 - stride_y * 2.0],
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
