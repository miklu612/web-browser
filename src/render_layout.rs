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
#[derive(Debug, Clone)]
pub struct Word {
    pub word: String,
    pub position: Position,
    pub width: i32,
    pub height: i32,
}

impl Word {
    pub fn new(word: String, position: Position, width: i32, height: i32) -> Self {
        Self {
            word,
            position,
            width,
            height,
        }
    }
}

impl Word {
    pub fn make_relative_to(&mut self, position: Position) {
        self.position = self.position + position;
    }

    /// Checks if a given position is inside of this word
    pub fn is_position_inside(&self, x: i32, y: i32) -> bool {
        !(x < self.position.x
            || y < self.position.y
            || x > self.position.x + self.width
            || y > self.position.y + self.height)
    }
}

/// A container for multiple words. Not a single sentence as the name would imply. These are needed
/// to apply different styles and functionality to different sections in a single paragraph
#[derive(Debug, Clone)]
pub struct Sentence {
    pub words: Vec<Word>,
    pub href: Option<String>,
    pub text_color: Option<Color>,
}

impl Sentence {
    pub fn make_relative_to(&mut self, position: Position) {
        for word in &mut self.words {
            word.make_relative_to(position);
        }
    }

    /// Checks if a given position is inside a word of this sentence
    pub fn is_position_inside(&self, x: i32, y: i32) -> bool {
        for word in &self.words {
            if word.is_position_inside(x, y) {
                return true;
            }
        }
        false
    }
}

/// A container for multiple sentences.
#[derive(Debug, Clone)]
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

    pub fn combine_sentences(&mut self, paragraph: Paragraph) {
        self.sentences.extend(paragraph.sentences);
    }
}

#[derive(Debug, Clone)]
enum Definition {
    Paragraph(ParagraphDefinition),
    Table(TableDefinition),
}

/// A definition of an element rect that has not been created yet. This is a part of the
/// preprocessing step and will be turned into a rect later on.
#[derive(Debug)]
pub struct ElementDefinition {
    pub tag: Tag,
    pub children: Vec<Definition>,
}

/// Connects the paragraphs of these elements. Returns the remainder elements of child if there is
/// any.
pub fn connect_paragraphs(
    parent: &mut ElementDefinition,
    mut child: ElementDefinition,
) -> Option<ElementDefinition> {
    match parent.children.last_mut() {
        Some(Definition::Paragraph(parent_paragraph)) => {
            let mut iter = child.children.iter().peekable();
            loop {
                match iter.peek() {
                    Some(Definition::Paragraph(paragraph)) => {
                        iter.next();
                        parent_paragraph
                            .sentences
                            .extend(paragraph.sentences.clone());
                    }
                    None => break,
                    _ => todo!(),
                }
            }
            child.children = iter.map(|x| x.clone()).collect();
            if child.children.len() == 0 {
                None
            } else {
                Some(child)
            }
        }

        _ => Some(child),
    }
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
            definition.children.push(Definition::Paragraph(paragraph));
            allow_paragraph_connecting = true;
        } else if child.element_type == Tag::Span || child.element_type == Tag::A {
            let child_definition = collect_definition(child);
            if allow_paragraph_connecting {
                if let Some(remaining_children) =
                    connect_paragraphs(&mut definition, child_definition)
                {
                    definition.children.extend(remaining_children.children);
                }
            } else {
                let child_definition = collect_definition(child);
                definition.children.extend(child_definition.children);
            }
            allow_paragraph_connecting = true;
        } else if child.element_type == Tag::Table {
            println!("TABLE");
            let table = TableDefinition::from_element(&child).unwrap();
            definition.children.push(Definition::Table(table));
        } else {
            let child_definition = collect_definition(child);
            definition.children.extend(child_definition.children);
            allow_paragraph_connecting = false;
        }
    }
    definition
}

#[derive(Clone, Debug)]
pub struct SentenceDefinition {
    pub tag: Tag,
    pub words: Vec<String>,
    pub href: Option<String>,
    pub text_color: Option<Color>,
}

impl SentenceDefinition {
    pub fn as_string(&self) -> String {
        let mut sentence = String::new();
        for word in &self.words {
            sentence += word;
        }
        sentence
    }
}

/// A collection of elements that should be drawn inline
#[derive(Debug, Clone)]
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
        let text_color = match element.element_type {
            Tag::A => Some(Color::blue()),
            _ => None,
        };
        Self {
            tag: element.element_type,
            sentences: vec![SentenceDefinition {
                words,
                tag: element.element_type,
                href: element.get_attribute("href"),
                text_color,
            }],
            font_size: match element.element_type {
                Tag::H(1) => DEFAULT_H1_SIZE,
                _ => DEFAULT_FONT_SIZE,
            },
            background_color: None,
        }
    }

    /// Returns the width of this paragraph if all of the sentences were to be placed inline
    pub fn get_width(&self, font: &Font) -> i32 {
        let mut length = 0;
        for sentence in &self.sentences {
            length += font.get_word_width(&sentence.as_string(), DEFAULT_FONT_SIZE * 2.0);
        }
        length
    }

    /// Returns a compiled version of this paragraph. The output hasn't yet been given a position,
    /// so it will be positioned at 0, 0
    pub fn compile(&self, viewport_size: Size, font: &Font) -> Paragraph {
        let seperation_width = 10;
        let seperation_height = font.get_glyph_height(self.font_size);
        let mut x_position: i32 = 0;
        let mut sentences = Vec::new();
        let mut y_position: i32 = 0;

        for sentence in &self.sentences {
            let mut words = Vec::new();
            for word in &sentence.words {
                let word_width = font.get_word_width(word, self.font_size);
                let word_height = font.get_glyph_height(self.font_size);
                let mut right_edge = x_position + word_width;
                if right_edge > viewport_size.width {
                    y_position += seperation_height;
                    x_position = 0;
                    right_edge = font.get_word_width(word, self.font_size);
                }
                words.push(Word::new(
                    word.clone(),
                    Position::new(x_position, y_position),
                    word_width,
                    word_height,
                ));
                x_position = right_edge + seperation_width;
            }
            sentences.push(Sentence {
                words,
                href: sentence.href.clone(),
                text_color: sentence.text_color,
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

#[derive(Debug, Clone)]
pub struct TableRowDefinition {
    values: Vec<ParagraphDefinition>,
}

impl TableRowDefinition {
    pub fn from_element(element: &Element) -> Result<Self, String> {
        if element.element_type != Tag::Tr {
            return Err("Expected tag 'tr'".to_string());
        }

        let mut values = Vec::new();
        for child in &element.children {
            if child.element_type != Tag::Td {
                return Err("Expected tag 'td'".to_string());
            }
            let definition = collect_definition(child);
            assert!(
                definition.children.len() == 1,
                "Only one value inside of tables are supported for now"
            );
            match definition.children.first().unwrap() {
                Definition::Paragraph(v) => values.push(v.clone()),

                _ => {
                    return Err(
                        "Trying to render a non-paragraph element inside a table".to_string()
                    )
                }
            }
        }

        Ok(Self { values })
    }
}

/// Stores information needed to create a table. This needs to be a unique struct due to the table
/// element's unique formatting rules.
#[derive(Debug, Clone)]
pub struct TableDefinition {
    rows: Vec<TableRowDefinition>,
}

pub struct Table {
    paragraphs: Vec<Paragraph>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            paragraphs: Vec::new(),
        }
    }
}

impl TableDefinition {
    pub fn from_element(element: &Element) -> Result<Self, String> {
        if element.element_type != Tag::Table {
            return Err(format!("Expected table. Got: '{:?}'", element.element_type));
        }

        let mut rows = Vec::new();
        for child in &element.children {
            let row = TableRowDefinition::from_element(child).unwrap();
            rows.push(row);
        }
        Ok(Self { rows })
    }

    /// Compile this table into a rendeable [Table]
    pub fn compile(&self, viewport_size: Size, font: &Font) -> Table {
        println!("Compiling {} table values", self.rows.len());

        // Calculate the column widths so the elements can be placed correctly
        let mut max_column_widths = Vec::new();
        for row in &self.rows {
            for (index, sentence) in row.values.iter().enumerate() {
                let sentence_length = sentence.get_width(font);
                if let Some(v) = max_column_widths.get_mut(index) {
                    *v = sentence_length;
                } else {
                    max_column_widths.push(sentence_length);
                }
            }
        }

        // Compile into paragraphs
        let mut output = Table::new();
        let mut y = 0;
        for row in &self.rows {
            for (column_index, column) in row.values.iter().enumerate() {
                // Get the x position for this column element
                let mut x_position = 0;
                for i in 0..column_index {
                    x_position += max_column_widths[i];
                }

                let mut paragraph = column.compile(Size::new(2000, 2000), font);
                paragraph.make_relative_to(Position::new(x_position, y));
                output.paragraphs.push(paragraph);
            }
            y += DEFAULT_FONT_SIZE as i32;
        }

        output
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
            for child_definition in definition.children {
                match child_definition {
                    Definition::Paragraph(paragraph) => {
                        paragraphs.push(paragraph.compile(viewport_size, font))
                    }

                    Definition::Table(table) => {
                        let table_values = table.compile(viewport_size, font);
                        let mut paragraph = table_values.paragraphs.first().unwrap().clone();
                        for i in 1..table_values.paragraphs.len() {
                            paragraph.combine_sentences(table_values.paragraphs[i].clone());
                        }
                        paragraphs.push(paragraph);
                    }

                    v => panic!("Not Implemented For '{:?}'", v),
                }
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
