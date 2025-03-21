use crate::bound::Bound;
use ab_glyph::{point, Font as AbFont, FontVec, ScaleFont};
use glium::Texture2d;
use image::{Rgba, RgbaImage};
use std::{collections::HashMap, fs::File, io::Read, path::Path};

pub struct Font {
    glyphs: HashMap<char, Texture2d>,
    pub font: FontVec,
}

impl Font {
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = match File::open(path) {
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };

        let bytes = file.bytes();
        let bytes: Vec<u8> = bytes.map(|x| x.unwrap()).collect();
        println!("{}", bytes.len());

        let font = match FontVec::try_from_vec(bytes) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}", e)),
        };

        Ok(Self {
            glyphs: HashMap::new(),
            font,
        })
    }

    pub fn get_glyph_width(&self, character: char, font_size: f32) -> i32 {
        let glyph = self.font.glyph_id(character);
        self.font.as_scaled(font_size).h_advance(glyph) as i32
    }

    pub fn get_glyph_height(&self, font_size: f32) -> i32 {
        self.font.as_scaled(font_size).height() as i32
    }

    pub fn get_glyph_bounds(&self, character: char, font_size: f32) -> Bound<i32> {
        let font_scaled = self.font.as_scaled(font_size);
        let glyph = self.font.glyph_id(character).with_scale(font_size);
        Bound::<i32>::new(
            self.get_glyph_width(character, font_size),
            font_scaled.height() as i32,
        )
    }

    pub fn get_word_width(&self, word: &str, font_size: f32) -> i32 {
        let mut width = 0;
        for character in word.chars() {
            width += self.get_glyph_bounds(character, font_size).width;
        }
        width
    }

    pub fn render_string(&self, word: &str, font_size: f32) -> RgbaImage {
        let mut output = RgbaImage::new(
            self.get_word_width(word, font_size) as u32 + 1,
            self.get_glyph_height(font_size) as u32 + 1,
        );

        let mut previous_point = point(0.0, self.font.as_scaled(font_size).ascent());
        for character in word.chars() {
            let glyph = self
                .font
                .glyph_id(character)
                .with_scale_and_position(font_size, previous_point);
            if let Some(outline) = self.font.outline_glyph(glyph) {
                let bounding_box = outline.px_bounds();
                outline.draw(|x, y, c| {
                    if c > 0.0 {
                        output.put_pixel(
                            x + bounding_box.min.x as u32,
                            (y as i32 + bounding_box.min.y as i32) as u32,
                            Rgba([0, 0, 0, (255.0 * c) as u8]),
                        );
                    }
                });
            }
            previous_point.x += self.get_glyph_width(character, font_size) as f32;
        }
        output
    }
}
