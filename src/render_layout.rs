//! This module handles the layout of HTML document when it is rendered onto the screen.
//!
//! This system works with rectangles. A rectangle is given a width and a height which will tell
//! how the elements fit inside of the rectangle. This is done automatically. All children
//! positions are relative to the parent. And their children are relative to their parent all the
//! way down until there are no children.

use crate::html::{Element, Tag};
use std::ops::Add;

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
pub struct Sentence {
    pub words: Vec<Word>,
}

impl Sentence {
    pub fn make_relative_to(&mut self, position: Position) {
        for word in &mut self.words {
            word.make_relative_to(position);
        }
    }
}

/// A container for multiple sentences.
pub struct Paragraph {
    pub sentences: Vec<Sentence>,
    pub height: i32,
    pub font_scale: f32,
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

impl ElementDefinition {}

/// Collects the different element definitions from the element
pub fn collect_definition(element: &Element) -> ElementDefinition {
    let mut definition = ElementDefinition {
        tag: element.element_type,
        children: Vec::new(),
    };
    let mut allow_paragraph_connecting = false;

    for child in &element.children {
        if child.element_type == Tag::PlainText {
            definition.children.push(ParagraphDefinition::from_string(
                element.element_type,
                &child.inner_text,
            ));
            allow_paragraph_connecting = true;
        } else if child.element_type == Tag::Span || child.element_type == Tag::A {
            let child_definition = collect_definition(child);
            if allow_paragraph_connecting {
                for child_paragraph in child_definition.children {
                    if let Some(last) = definition.children.last_mut() {
                        last.words.extend(child_paragraph.words.clone());
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

/// A collection of elements that should be drawn inline
pub struct ParagraphDefinition {
    pub tag: Tag,
    pub words: Vec<String>,
    pub font_scale: f32,
}

impl ParagraphDefinition {
    pub fn from_string(tag: Tag, string: &str) -> Self {
        let words = string.split(" ").map(|x| x.to_owned()).collect();
        Self {
            tag,
            words,
            font_scale: match tag {
                Tag::H(1) => 2.0,
                _ => 1.0,
            },
        }
    }

    /// Returns a compiled version of this paragraph. The output hasn't yet been given a position,
    /// so it will be positioned at 0, 0
    pub fn compile(self, viewport_size: Size, mut font_size: Size) -> Paragraph {
        let mut words = Vec::new();
        let mut x_position: i32 = 0;
        let mut y_position: i32 = 0;
        font_size.width = (font_size.width as f32 * self.font_scale) as i32;
        font_size.height = (font_size.height as f32 * self.font_scale) as i32;

        for word in self.words {
            let mut right_edge = x_position + word.len() as i32 * font_size.width;
            if right_edge > viewport_size.width {
                y_position += font_size.height;
                x_position = 0;
                right_edge = word.len() as i32 * font_size.width;
            }
            words.push(Word::new(word, Position::new(x_position, y_position)));
            x_position = right_edge + font_size.width;
        }

        let sentences = vec![Sentence { words }];

        Paragraph {
            sentences,
            height: y_position + font_size.height,
            font_scale: self.font_scale,
        }
    }
}

/// Holds the layout of the html
pub struct Layout {
    pub paragraphs: Vec<Paragraph>,
}

impl Layout {
    pub fn make_relative_to(&mut self, position: Position) {
        for paragraph in &mut self.paragraphs {
            paragraph.make_relative_to(position);
        }
    }

    pub fn from_body(element: &Element, viewport_size: Size, font_size: Size) -> Self {
        let mut definitions = Vec::new();

        for child in &element.children {
            definitions.push(collect_definition(child));
        }

        // Collect the paragraphs
        let mut paragraphs = Vec::new();
        for definition in definitions {
            for paragraph in definition.children {
                paragraphs.push(paragraph.compile(viewport_size, font_size));
            }
        }

        // Adjust the positions of the paragraphs
        let mut current_y = 0;
        let spacing = 50;
        for paragraph in &mut paragraphs {
            paragraph.make_relative_to(Position::new(0, current_y));
            current_y += paragraph.height;
            current_y += spacing;
        }

        Self { paragraphs }
    }
}
