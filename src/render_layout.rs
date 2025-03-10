//! This module handles the layout of HTML document when it is rendered onto the screen.

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
    width: i32,
    height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

/// A container for individual words. Contains the word and the position of it relative to the
/// parent [ElementRect]
#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub position: Position,
}

/// A rectangle that holds an element. Used in scaling and positioning
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
            let mut children = Vec::new();
            let mut last_position = position;
            for child in &element.children {
                let rect = ElementRect::from_element(&child, last_position, size, font_size);
                last_position.y += rect.get_height();
                children.push(rect);
            }
            ElementRect {
                position,
                size,
                font_size,
                tag: element.element_type,
                words: None,
                children: children,
            }
        }
    }

    pub fn get_height(&self) -> i32 {
        let mut height = 0;
        if let Some(words) = self.words.as_ref() {
            for word in words {
                if word.position.y + 20 > height {
                    height = word.position.y - self.position.y + 20;
                }
            }
        }
        for child in &self.children {
            height = i32::max(height, child.get_height());
        }
        height
    }

    pub fn from_text(
        tag: Tag,
        text: &str,
        position: Position,
        size: Size,
        font_size: Size,
    ) -> Self {
        let words = text.split(" ").map(|x| x.trim().to_uppercase());

        let mut element_words = Vec::new();

        let mut current_x = position.x;
        let mut current_y = position.y;
        for word in words {
            if current_x > size.width {
                current_x = position.x;
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
