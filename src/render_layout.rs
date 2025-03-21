//! This module handles the layout of HTML document when it is rendered onto the screen.
//!
//! This system works with rectangles. A rectangle is given a width and a height which will tell
//! how the elements fit inside of the rectangle. This is done automatically. All children
//! positions are relative to the parent. And their children are relative to their parent all the
//! way down until there are no children.

use crate::color::Color;
use crate::css::{Color as CssColor, Rule};
use crate::font::Font;
use crate::html::{Element, Tag};
use std::ops::Add;

const DEFAULT_FONT_SIZE: f32 = 40.0;
const DEFAULT_H1_SIZE: f32 = DEFAULT_FONT_SIZE * 2.0;

/// A generic position vector implementation
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Add for Position {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A generic size vector implementation
#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn scaled(&self, scale: f32) -> Self {
        Self {
            width: (self.width as f32 * scale) as i32,
            height: (self.height as f32 * scale) as i32,
        }
    }
}

/// A container for individual words.
#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub position: Position,
}

impl Word {
    pub fn new(word: String, position: Position) -> Self {
        Self { word, position }
    }
}

impl Word {
    pub fn make_relative_to(&mut self, position: Position) {
        self.position = self.position + position;
    }
}

/// A container for multiple words. Not a single sentence as the name would imply. These are needed
/// to apply different styles and functionality to different sections in a single paragraph
#[derive(Debug)]
pub struct Sentence {
    pub words: Vec<Word>,
    pub href: Option<String>,
}

impl Sentence {
    pub fn make_relative_to(&mut self, position: Position) {
        for word in &mut self.words {
            word.make_relative_to(position);
        }
    }
}

/// A container for multiple sentences.
#[derive(Debug)]
pub struct Paragraph {
    pub sentences: Vec<Sentence>,
    pub height: i32,
    pub font_size: f32,
    pub background_color: Option<Color>,
}

impl Paragraph {
    pub fn make_relative_to(&mut self, position: Position) {
        for sentence in &mut self.sentences {
            sentence.make_relative_to(position);
        }
    }
}

/// A definition of an element rect that has not been created yet. This is a part of the
/// preprocessing step and will be turned into a rect later on.
pub struct ElementDefinition {
    pub tag: Tag,
    pub children: Vec<ParagraphDefinition>,
}

/// Collects the different element definitions from the element
pub fn collect_definition(element: &Element) -> ElementDefinition {
    let mut definition = ElementDefinition {
        tag: element.element_type,
        children: Vec::new(),
    };
    let mut allow_paragraph_connecting = false;

    // Handle some css stuff
    let mut background_color = None;
    for rule in &element.inner_styles {
        if let Rule::BackgroundColor(color) = rule {
            match color {
                CssColor::Hex(r, g, b) => {
                    background_color = Some(Color {
                        r: *r as f32 / 255.0,
                        g: *g as f32 / 255.0,
                        b: *b as f32 / 255.0,
                        a: 1.0,
                    });
                }
                _ => panic!("Color not implemented"),
            }
        }
    }

    for child in &element.children {
        if child.element_type == Tag::PlainText {
            let mut paragraph = ParagraphDefinition::from_string(element, &child.inner_text);
            paragraph.background_color = background_color;
            definition.children.push(paragraph);
            allow_paragraph_connecting = true;
        } else if child.element_type == Tag::Span || child.element_type == Tag::A {
            let child_definition = collect_definition(child);
            if allow_paragraph_connecting {
                for child_paragraph in child_definition.children {
                    if let Some(last) = definition.children.last_mut() {
                        last.sentences.extend(child_paragraph.sentences.clone());
                    } else {
                        definition.children.push(child_paragraph);
                    }
                }
            } else {
                let child_definition = collect_definition(child);
                definition.children.extend(child_definition.children);
            }
            allow_paragraph_connecting = true;
        } else {
            let child_definition = collect_definition(child);
            definition.children.extend(child_definition.children);
            allow_paragraph_connecting = false;
        }
    }
    definition
}

#[derive(Clone)]
pub struct SentenceDefinition {
    pub tag: Tag,
    pub words: Vec<String>,
    pub href: Option<String>,
}

/// A collection of elements that should be drawn inline
pub struct ParagraphDefinition {
    pub tag: Tag,
    pub sentences: Vec<SentenceDefinition>,
    pub font_size: f32,
    pub background_color: Option<Color>,
}

impl ParagraphDefinition {
    /// Create an element with the string as its content
    ///
    /// * `element` - The parent element of this string. The inner content is not used from this,
    ///               but the attributes are used.
    ///
    /// * `string` - The content this element contains
    pub fn from_string(element: &Element, string: &str) -> Self {
        let words = string.split(" ").map(|x| x.to_owned()).collect();
        Self {
            tag: element.element_type,
            sentences: vec![SentenceDefinition {
                words,
                tag: element.element_type,
                href: element.get_attribute("href"),
            }],
            font_size: match element.element_type {
                Tag::H(1) => DEFAULT_H1_SIZE,
                _ => DEFAULT_FONT_SIZE,
            },
            background_color: None,
        }
    }

    /// Returns a compiled version of this paragraph. The output hasn't yet been given a position,
    /// so it will be positioned at 0, 0
    pub fn compile(self, viewport_size: Size, font: &Font) -> Paragraph {
        let seperation_width = 10;
        let seperation_height = font.get_glyph_height(self.font_size);
        let mut x_position: i32 = 0;
        let mut sentences = Vec::new();
        let mut y_position: i32 = 0;

        for sentence in &self.sentences {
            let mut words = Vec::new();
            for word in &sentence.words {
                let mut right_edge = x_position + font.get_word_width(word, self.font_size);
                if right_edge > viewport_size.width {
                    y_position += seperation_height;
                    x_position = 0;
                    right_edge = font.get_word_width(word, self.font_size);
                }
                words.push(Word::new(
                    word.clone(),
                    Position::new(x_position, y_position),
                ));
                x_position = right_edge + seperation_width;
            }
            sentences.push(Sentence {
                words,
                href: sentence.href.clone(),
            });
        }

        Paragraph {
            sentences,
            height: y_position + seperation_height,
            font_size: self.font_size,
            background_color: self.background_color,
        }
    }
}

/// Holds the layout of the html
#[derive(Debug)]
pub struct Layout {
    pub paragraphs: Vec<Paragraph>,
}

impl Layout {
    pub fn make_relative_to(&mut self, position: Position) {
        for paragraph in &mut self.paragraphs {
            paragraph.make_relative_to(position);
        }
    }

    pub fn from_body(element: &Element, viewport_size: Size, font: &Font) -> Self {
        let mut definitions = Vec::new();

        for child in &element.children {
            definitions.push(collect_definition(child));
        }

        // Collect the paragraphs
        let mut paragraphs = Vec::new();
        for definition in definitions {
            for paragraph in definition.children {
                paragraphs.push(paragraph.compile(viewport_size, font));
            }
        }

        // Adjust the positions of the paragraphs
        let mut current_y = 0;
        let spacing: i32 = (font.get_glyph_height(DEFAULT_FONT_SIZE) as f32 / 2.0) as i32;
        for paragraph in &mut paragraphs {
            paragraph.make_relative_to(Position::new(0, current_y));
            current_y += paragraph.height;
            current_y += spacing;
        }

        Self { paragraphs }
    }
}
