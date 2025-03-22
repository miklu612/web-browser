use crate::color::Color;
use crate::document::Document;
use crate::font::Font;
use crate::html::{parse_html, Element, Tag};
use crate::render_layout::{Layout, Position, Size};
use crate::requests::get_site;
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
use image::RgbaImage;
use nalgebra::{Matrix4, Vector3};
use std::{num::NonZero, path::Path};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::Key,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Cursor, CursorIcon, Window as WinitWindow, WindowId},
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
    mouse_position: Position,
    display: Option<Display<WindowSurface>>,
    rect: Option<Rectangle>,
    program: Option<Program>,
    document: Option<Document>,
    scroll_y: i32,
    font: Option<Font>,
    layout: Option<Layout>,
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
                    uniform vec4 background_color;
                    void main() {
                        color = texture(font_texture, texCoord);
                        color.x = color.x;
                        color.y = color.y;
                        color.z = color.z;

                        // 0.1 is like a toggle switch for the time being
                        if(color.a < 0.1 && background_color.a > 0.1) {
                            color = background_color;
                        }
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
                Key::Character("j") => self.scroll_y -= 10,
                Key::Character("k") => self.scroll_y += 10,
                _ => (),
            },

            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                self.update_cursor(x as i32, y as i32);
            }

            WindowEvent::MouseInput { button, state, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    self.handle_click();
                }
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
            scroll_y: 0,
            document: None,
            font: None,
            layout: None,
            mouse_position: Position::new(0, 0),
        }
    }

    /// Transforms screen coordinates into a -1.0 - 1.0 scale
    pub fn screen_to_opengl_coordinates(&self, x: i32, y: i32) -> [f32; 2] {
        let inner_size = self.window.as_ref().unwrap().inner_size();
        [
            (x as f32 / inner_size.width as f32 - 0.5) * 2.0,
            (y as f32 / inner_size.height as f32 - 0.5) * -2.0,
        ]
    }

    pub fn update_cursor(&mut self, x: i32, y: i32) {
        self.mouse_position.x = x;
        self.mouse_position.y = y;
        if let Some(layout) = self.layout.as_ref() {
            let mut cursor_mode = CursorIcon::Default;
            for paragraph in &layout.paragraphs {
                for sentence in &paragraph.sentences {
                    if sentence.href.is_some() {
                        if sentence.is_position_inside(x, y - self.scroll_y) {
                            cursor_mode = CursorIcon::Pointer;
                            break;
                        }
                    }
                }
            }
            self.window
                .as_ref()
                .unwrap()
                .set_cursor(Cursor::Icon(cursor_mode));
        }
    }

    pub fn handle_click(&mut self) {
        if let Some(layout) = self.layout.as_ref() {
            let x = self.mouse_position.x;
            let y = self.mouse_position.y;
            let mut cursor_mode = CursorIcon::Default;
            let mut new_elements = None;
            for paragraph in &layout.paragraphs {
                for sentence in &paragraph.sentences {
                    if sentence.href.is_some() {
                        if sentence.is_position_inside(x, y - self.scroll_y) {
                            println!("Getting {:?}", sentence.href);
                            new_elements =
                                Some(parse_html(&get_site(&sentence.href.as_ref().unwrap())));
                            println!("Content received!");
                        }
                    }
                }
            }
            if let Some(elements) = new_elements.take() {
                self.set_elements(elements);
            }
            self.window
                .as_ref()
                .unwrap()
                .set_cursor(Cursor::Icon(cursor_mode));
        }
    }

    /// Transforms screen coordinates into a 0.0 - 1.0 scale
    pub fn screen_to_relative_coordinates(&self, x: i32, y: i32) -> [f32; 2] {
        let inner_size = self.window.as_ref().unwrap().inner_size();
        [
            x as f32 / inner_size.width as f32,
            y as f32 / inner_size.height as f32,
        ]
    }

    pub fn set_elements(&mut self, elements: Vec<Element>) {
        self.document = Some(Document::new(elements, Vec::new()));
        self.document.as_mut().unwrap().parse_inline_css();
    }

    pub fn render(&mut self, elements: Vec<Element>) {
        self.set_elements(elements);
        self.open();
    }

    pub fn render_string(
        &self,
        frame: &mut Frame,
        string: &str,
        x: i32,
        y: i32,
        font_size: f32,
        background_color: Option<Color>,
        text_color: Color,
    ) {
        // Culling
        let top_y = self.screen_to_opengl_coordinates(0, y)[1];
        let bottom_y = self.screen_to_opengl_coordinates(
            0,
            y + self.font.as_ref().unwrap().get_glyph_height(font_size),
        )[1];
        if bottom_y > 1.0 || top_y < -1.0 {
            return;
        }

        let rgba_image = self
            .font
            .as_ref()
            .unwrap()
            .render_string(string, font_size, text_color);
        let texture = self.rgba_image_to_texture(&rgba_image);

        let size = self.screen_to_relative_coordinates(
            (rgba_image.width()) as i32,
            (rgba_image.height()) as i32,
        );

        let gl_coordinates = self.screen_to_opengl_coordinates(
            x + (rgba_image.width() as f32 / 2.0) as i32,
            y + (rgba_image.height() as f32 / 2.0) as i32,
        );
        let mat4 = Matrix4::identity()
            .append_nonuniform_scaling(&Vector3::new(size[0], size[1], 1.0))
            .append_translation(&Vector3::new(gl_coordinates[0], gl_coordinates[1], 0.0));
        let compiled_matrix = TryInto::<[[f32; 4]; 4]>::try_into(mat4.data.0).unwrap();

        let bg_color = match background_color {
            Some(color) => [color.r, color.g, color.b, 1.0],
            None => [0.0, 0.0, 0.0, 0.0],
        };

        let uniforms = uniform![
            transform: compiled_matrix,
            font_texture: texture,
            background_color: bg_color
        ];

        frame
            .draw(
                &self.rect.as_ref().unwrap().vao,
                &self.rect.as_ref().unwrap().ebo,
                self.program.as_ref().unwrap(),
                &uniforms,
                &DrawParameters {
                    blend: Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
    }

    pub fn render_current_page(&mut self, frame: &mut Frame) {
        self.update_page_layout();
        for paragraph in &self.layout.as_ref().unwrap().paragraphs {
            for sentence in &paragraph.sentences {
                let color = match sentence.text_color {
                    Some(v) => v,
                    None => Color::black(),
                };
                for word in &sentence.words {
                    self.render_string(
                        frame,
                        &word.word,
                        word.position.x,
                        word.position.y + self.scroll_y,
                        paragraph.font_size,
                        paragraph.background_color,
                        color,
                    );
                }
            }
        }
    }

    pub fn update_page_layout(&mut self) {
        let mut body = None;
        for element in &self.document.as_ref().unwrap().elements[0].children {
            if element.element_type == Tag::Body {
                body = Some(element);
                break;
            }
        }
        let body = body.unwrap();
        let inner_size = self.window.as_ref().unwrap().inner_size();
        let mut layout = Layout::from_body(
            body,
            Size {
                width: inner_size.width as i32 - 40,
                height: inner_size.height as i32 - 40,
            },
            self.font.as_ref().unwrap(),
        );
        layout.make_relative_to(Position::new(40, 40));
        self.layout = Some(layout);
    }

    pub fn rgba_image_to_texture(&self, image: &RgbaImage) -> Texture2d {
        let dimensions = image.dimensions();
        let raw_image = RawImage2d::from_raw_rgba_reversed(&image.clone().into_raw(), dimensions);
        Texture2d::new(self.display.as_ref().unwrap(), raw_image).unwrap()
    }

    pub fn load_font(&mut self) {
        self.font = Some(
            Font::load(Path::new(
                "./fonts/liberation-sans/LiberationSans-Regular.ttf",
            ))
            .unwrap(),
        );
    }

    pub fn open(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}
