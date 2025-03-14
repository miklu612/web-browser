//! This module handles the layout of HTML document when it is rendered onto the screen.
//!
//! This system works with rectangles. A rectangle is given a width and a height which will tell
//! how the elements fit inside of the rectangle. This is done automatically. All children
//! positions are relative to the parent. And their children are relative to their parent all the
//! way down until there are no children.

use crate::css::{Rule, Unit};
use crate::html::{Element, Tag};

/// A generic position vector implementation
#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
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

/// A container for individual words. Contains the word and the position of it relative to the
/// parent [ElementRect]
#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub position: Position,
}

/// A rectangle that holds an element. Used in scaling and positioning during rendering.
#[derive(Debug)]
pub struct ElementRect {
    pub position: Position,
    pub size: Size,
    pub font_size: Size,
    pub tag: Tag,
    pub words: Option<Vec<Word>>,
    pub children: Vec<ElementRect>,
}

impl ElementRect {
    /// Create a [ElementRect] from an [Element]
    ///
    /// * `element` the tag of this element
    /// * `position` the position of this element relative to parent
    /// * `size` the size of this element
    /// * `font_size` the font size of this element
    pub fn from_element(
        element: &Element,
        position: Position,
        size: Size,
        font_size: Size,
    ) -> ElementRect {
        if element.element_type == Tag::PlainText {
            ElementRect::from_text(
                Tag::PlainText,
                &element.inner_text,
                position,
                size,
                font_size,
            )
        } else {
            let mut new_font_size = if element.element_type == Tag::H(1) {
                font_size.scaled(2.0)
            } else {
                font_size
            };
            let mut children = Vec::new();
            let mut last_position = Position::new(0, 0);

            // Search for position altering css rules
            for rule in &element.inner_styles {
                match rule {
                    Rule::MarginLeft(v) => match v {
                        Unit::Px(pixels) => {
                            last_position.x += pixels;
                        }
                        _ => panic!("Unimplemented unit found"),
                    },
                    _ => {}
                }
            }

            for child in &element.children {
                let rect = ElementRect::from_element(&child, last_position, size, new_font_size);
                last_position.y += rect.get_height();
                children.push(rect);
            }
            ElementRect {
                position,
                size,
                font_size: new_font_size,
                tag: element.element_type,
                words: None,
                children: children,
            }
        }
    }

    /// Returns the height of this element
    pub fn get_height(&self) -> i32 {
        let mut height = 0;
        if let Some(words) = self.words.as_ref() {
            for word in words {
                if word.position.y + self.font_size.height > height {
                    height = word.position.y + self.font_size.height;
                }
            }
        }
        let mut child_height = 0;
        for child in &self.children {
            child_height += child.get_height();
        }
        i32::max(height, child_height)
    }

    /// Create a [ElementRect] from text content
    ///
    /// * `tag` the tag of this element
    /// * `text` the text of this element
    /// * `position` the position of this element relative to parent
    /// * `size` the size of this element
    /// * `font_size` the font size of this element
    pub fn from_text(
        tag: Tag,
        text: &str,
        position: Position,
        size: Size,
        font_size: Size,
    ) -> Self {
        let words = text.split(" ").map(|x| x.trim().to_uppercase());

        let mut element_words = Vec::new();

        let mut current_x = 0;
        let mut current_y = 0;
        for word in words {
            if current_x + word.len() as i32 * font_size.width > size.width {
                current_x = 0;
                current_y += font_size.height;
            }
            element_words.push(Word {
                word: word.to_owned(),
                position: Position::new(current_x, current_y),
            });
            // Add a space and calculate the next word offset
            current_x += font_size.width;
            let x_offset = word.len() as i32 * font_size.width;
            current_x += x_offset;
        }

        Self {
            position,
            tag,
            size,
            font_size,
            words: Some(element_words),
            children: Vec::new(),
        }
    }
}
