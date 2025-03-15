//! This module handles the layout of HTML document when it is rendered onto the screen.
//!
//! This system works with rectangles. A rectangle is given a width and a height which will tell
//! how the elements fit inside of the rectangle. This is done automatically. All children
//! positions are relative to the parent. And their children are relative to their parent all the
//! way down until there are no children.

use crate::css::{Rule, Unit};
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

/// A container for individual words. Contains the word and the position of it relative to the
/// parent [ElementRect]
#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub position: Position,
}

/// The direction where the current tree of nodes will grow towards. This will usually be in the
/// down and right directions respectiply
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ExpandDirection {
    Vertical,
    Horizontal(Position),
}

/// A rectangle that holds an element. Used in scaling and positioning during rendering.
#[derive(Debug)]
pub struct ElementRect {
    pub position: Position,
    pub global_position: Position,
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
        global_position: Position,
        size: Size,
        font_size: Size,
        mut expand_direction: ExpandDirection,
    ) -> ElementRect {
        if element.element_type == Tag::PlainText {
            ElementRect::from_text(
                Tag::PlainText,
                &element.inner_text,
                position,
                global_position,
                size,
                font_size,
                expand_direction,
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

            let mut is_original_setter = false;
            if element.element_type == Tag::Span && expand_direction == ExpandDirection::Vertical {
                expand_direction = ExpandDirection::Horizontal(global_position);
                is_original_setter = true;
            }

            for child in &element.children {
                let mut rect = ElementRect::from_element(
                    &child,
                    last_position,
                    global_position + last_position,
                    Size::new(size.width - last_position.x, size.height - last_position.y),
                    new_font_size,
                    expand_direction,
                );
                match expand_direction {
                    ExpandDirection::Vertical => last_position.y += rect.get_height(),

                    ExpandDirection::Horizontal(start_global_position) => {
                        last_position = rect.position + rect.get_next_horizontal_position();
                        last_position.x += font_size.width;
                        if last_position.x > size.width {
                            last_position.x = start_global_position.x - global_position.x;
                            last_position.y += font_size.height;
                        }
                    }
                }
                children.push(rect);
            }
            ElementRect {
                position,
                global_position,
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
        for child in &self.children {
            height = height.max(child.position.y + child.get_height());
        }
        height
    }

    pub fn get_right_edge(&self) -> i32 {
        let mut max_x = 0;
        if let Some(words) = self.words.as_ref() {
            for word in words {
                let len = word.position.x + self.font_size.width * word.word.len() as i32;
                max_x = i32::max(len, max_x);
            }
        }
        let mut child_width = 0;
        for child in &self.children {
            child_width = i32::max(child.position.x + child.get_right_edge(), child_width);
        }
        i32::max(max_x, child_width)
    }

    pub fn get_next_horizontal_position(&self) -> Position {
        let mut next_pos = Position::new(0, 0);

        if let Some(words) = self.words.as_ref() {
            for word in words {
                if word.position.y > next_pos.y {
                    next_pos = Position::new(
                        word.position.x + self.font_size.width * word.word.len() as i32,
                        word.position.y,
                    );
                } else if word.position.y == next_pos.y
                    && word.position.x + self.font_size.width * word.word.len() as i32 > next_pos.x
                {
                    next_pos = Position::new(
                        word.position.x + self.font_size.width * word.word.len() as i32,
                        word.position.y,
                    );
                }
            }
        }
        for child in &self.children {
            let child_next = child.get_next_horizontal_position() + child.position;
            if child_next.y > next_pos.y {
                next_pos = child_next;
            } else if child_next.y == next_pos.y && child_next.x > next_pos.x {
                next_pos = child_next;
            }
        }

        next_pos
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
        global_position: Position,
        outer_size: Size,
        font_size: Size,
        expand_direction: ExpandDirection,
    ) -> Self {
        let mut size = outer_size;

        // Just in case, so the program won't crash due to a divide-by-zero error.
        if size.width <= 0 {
            size.width = 1;
        }

        let words = text.split(" ").map(|x| x.trim().to_uppercase());

        let mut element_words = Vec::new();

        let mut current_x = 0;
        let mut current_y = 0;
        for word in words {
            let current_boundary_index = f32::floor(current_x as f32 / size.width as f32) as i32;
            let next_boundary_index = current_boundary_index + 1;
            let x_offset = word.len() as i32 * font_size.width + font_size.width;

            if current_x + x_offset > size.width * next_boundary_index {
                match expand_direction {
                    ExpandDirection::Horizontal(start_pos) => {
                        let delta = global_position.x - start_pos.x;
                        size.width = outer_size.width + delta;
                        current_x = next_boundary_index * size.width - delta;
                        current_y =
                            f32::floor((current_x + delta) as f32 / size.width as f32 as f32)
                                as i32
                                * font_size.height;
                    }
                    ExpandDirection::Vertical => {
                        current_y = next_boundary_index * font_size.height;
                        current_x = next_boundary_index * size.width;
                    }
                }
            }

            element_words.push(Word {
                word: word.to_owned(),
                position: Position::new(current_x % size.width, current_y),
            });
            current_x += x_offset;
        }

        Self {
            position,
            global_position,
            tag,
            size,
            font_size,
            words: Some(element_words),
            children: Vec::new(),
        }
    }
}
