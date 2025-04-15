//! A Css parser

use crate::html::Tag;
use std::{iter::Peekable, str::Chars};

/// These represent the different units that are used in CSS. Their names should be equivalent to
/// their css counterparts.
///
/// ## Example
///
/// `Unit::Px(50)` would represent the following CSS code.
///
/// ```css
/// 50px
/// ```
#[derive(Debug, Copy, Clone)]
pub enum Unit {
    Px(i32),
    Pt(i32),
    Em(f32),
    Percentage(i32),
}

/// This contains all of the different colours that css supports.
#[derive(Debug, Copy, Clone)]
pub enum Color {
    Hex(u8, u8, u8),
    White,
    Gray,
    Lightgray,
    Black,
    Transparent,
}

/// This contains all of the different values for border and such. For example
/// `border: 20px solid`
#[derive(Debug)]
pub enum BorderStyle {
    Solid,
}

#[derive(Debug)]
pub enum DisplayStyle {
    Block,
    None,
    Flex,
}

#[derive(Debug)]
pub enum Direction {
    Right,
    Left,
}

#[derive(Debug)]
pub enum FontStyle {
    Italic,
}

#[derive(Debug)]
pub enum FontWeight {
    Normal,
}

#[derive(Debug)]
pub enum WhiteSpace {
    NoWrap,
}

#[derive(Debug)]
pub enum Position {
    Absolute,
}

/// This won't be used in the final product, but it will be used to contain a variable value to
/// return in function calls. It will be up to the function implementation to validate that this
/// value contains the correct value.
#[derive(Debug)]
pub enum Value {
    Unit(Unit),
    Color(Color),
    BorderStyle(BorderStyle),
    DisplayStyle(DisplayStyle),
    Direction(Direction),
    FontStyle(FontStyle),
    WhiteSpace(WhiteSpace),
    FontWeight(FontWeight),
    Number(i32),
    Position(Position),
    Inherit,
}

impl Value {
    /// Checks for a lot of values and returns one if it matches. Else it panics. This could
    /// probably be an option in the future, but for now to keep things simple, we'll just write it
    /// like this.
    pub fn from_string(css_value: &str) -> Value {
        if css_value.ends_with("px") {
            let without_px_suffix = css_value.strip_suffix("px").unwrap();
            if without_px_suffix.chars().all(|x| x.is_numeric()) {
                return Value::Unit(Unit::Px(without_px_suffix.parse().unwrap()));
            }
        } else if css_value.ends_with("pt") {
            let without_pt_suffix = css_value.strip_suffix("pt").unwrap();
            if without_pt_suffix.chars().all(|x| x.is_numeric()) {
                return Value::Unit(Unit::Pt(without_pt_suffix.parse().unwrap()));
            }
        } else if css_value.ends_with("em") {
            let without_em_suffix = css_value.strip_suffix("em").unwrap();
            if let Ok(v) = without_em_suffix.parse::<f32>() {
                return Value::Unit(Unit::Em(v));
            }
        } else if css_value.ends_with("%") {
            let without_percentage = css_value.strip_suffix("%").unwrap();
            if let Ok(v) = without_percentage.parse::<i32>() {
                return Value::Unit(Unit::Percentage(v));
            }
        } else if css_value == "white" {
            return Value::Color(Color::White);
        } else if css_value == "gray" {
            return Value::Color(Color::Gray);
        } else if css_value == "lightgray" {
            return Value::Color(Color::Lightgray);
        } else if css_value == "solid" {
            return Value::BorderStyle(BorderStyle::Solid);
        } else if css_value == "transparent" {
            return Value::Color(Color::Transparent);
        } else if css_value == "block" {
            return Value::DisplayStyle(DisplayStyle::Block);
        } else if css_value == "right" {
            return Value::Direction(Direction::Right);
        } else if css_value == "left" {
            return Value::Direction(Direction::Left);
        } else if css_value == "none" {
            return Value::DisplayStyle(DisplayStyle::None);
        } else if css_value == "italic" {
            return Value::FontStyle(FontStyle::Italic);
        } else if css_value == "nowrap" {
            return Value::WhiteSpace(WhiteSpace::NoWrap);
        } else if css_value == "normal" {
            return Value::FontWeight(FontWeight::Normal);
        } else if css_value == "flex" {
            return Value::DisplayStyle(DisplayStyle::Flex);
        } else if css_value == "inherit" {
            return Value::Inherit;
        } else if css_value == "absolute" {
            return Value::Position(Position::Absolute);
        } else if css_value.starts_with('#') {
            println!("Hex colors aren't implemented yet!");
            return Value::Color(Color::Black);
        } else if css_value.chars().all(|x| x.is_numeric()) {
            return Value::Number(css_value.parse::<i32>().unwrap());
        }

        panic!("Couldn't convert '{}' into a css value", css_value);
    }
}

/// Represents a single rule in a ruleset block
///
/// ## Example
///
/// `Rule::Width(Unit::Px(50))` would represent the following CSS code.
///
/// ```css
/// width: 50px;
/// ```
#[derive(Debug)]
pub enum Rule {
    Width(Unit),
    MarginLeft(Unit),
    BackgroundColor(Color),
    FontSize(Unit),
}

impl Rule {
    pub fn new(identifier: &str, value: Vec<Value>) -> Option<Self> {
        match identifier {
            "width" => match value.first().unwrap() {
                Value::Unit(unit) => Some(Self::Width(*unit)),
                _ => panic!("Expected unit"),
            },

            "margin-left" => match value.first().unwrap() {
                Value::Unit(unit) => Some(Self::MarginLeft(*unit)),
                _ => panic!("Expected unit"),
            },

            "background-color" => match value.first().unwrap() {
                Value::Color(color) => Some(Self::BackgroundColor(*color)),
                _ => panic!("Expected a color"),
            },

            "font-size" => match value.first().unwrap() {
                Value::Unit(v) => Some(Self::FontSize(*v)),
                v => panic!("Expected unit. Got '{:?}'", v),
            },

            _ => {
                println!("Unknown css identifier: {}", identifier);
                None
            }
        }
    }
}

/// Represents a single selector in a ruleset block
///
/// ## Example
///
/// `Selector::Element(Tag::P)` would represent the following CSS code. Of course without the block
/// statement. That is only there to give context where the element lies relative to the entirety
/// of CSS.
///
/// ```css
/// p {};
/// ```
#[derive(Debug)]
pub enum Selector {
    Element(Tag),
}

impl Selector {
    pub fn new(selector: &str) -> Self {
        match selector {
            "p" => Selector::Element(Tag::Paragraph),

            _ => panic!("Unknown selector '{}'", selector),
        }
    }
}

/// Represents an entire ruleset block with selectors and rules.
///
/// ## Example
///
/// ```rust
/// RuleSet {
///     selectors: vec![Selector::Element(Tag::P)],
///     rules: vec![Rule::Width(Unit::Px(50))],
/// }
/// ```
///
/// would be equivalent to
///
/// ```css
/// p {
///     width: 50px;
/// }
/// ```
#[derive(Debug)]
pub struct RuleSet {
    pub selectors: Vec<Selector>,
    pub rules: Vec<Rule>,
}

impl RuleSet {
    pub fn new(selectors: Vec<Selector>, rules: Vec<Rule>) -> Self {
        Self { selectors, rules }
    }
}

pub fn skip_whitespace(iterator: &mut Peekable<Chars>) {
    loop {
        match iterator.peek() {
            Some(v) if v.is_whitespace() => {
                iterator.next();
            }
            _ => break,
        }
    }
}

pub fn get_identifier(iterator: &mut Peekable<Chars>) -> String {
    let mut output = String::new();
    loop {
        match iterator.peek() {
            Some(':') => break,

            Some('-') => {
                output.push(iterator.next().unwrap());
            }

            Some(v) if v.is_alphabetic() => {
                output.push(iterator.next().unwrap());
            }

            _ => break,
        }
    }
    output
}

/// The iterator has to begin at the first letter of the first value. The iterator will be returned
/// in the ending `;` character.
pub fn parse_css_value_unit(iterator: &mut Peekable<Chars>) -> Unit {
    let mut digit: i32 = 0;
    let output;
    loop {
        match iterator.peek() {
            Some(v) if v.is_numeric() => {
                digit = digit * 10 + v.to_digit(10).unwrap() as i32;
                iterator.next();
            }

            Some('p') => {
                assert_eq!(iterator.next(), Some('p'));
                assert_eq!(iterator.next(), Some('x'));
                output = Some(Unit::Px(digit));
                break;
            }

            _ => panic!("Unhandled character in value unit: '{:?}'", iterator.next()),
        }
    }
    output.expect("Couldn't evaluate value unit type")
}

pub fn collect_until_terminator(iterator: &mut Peekable<Chars>, terminators: &[char]) -> String {
    let mut output = String::new();
    loop {
        match iterator.peek() {
            Some(v) => {
                if !terminators.contains(v) {
                    output.push(*v);
                    iterator.next();
                } else {
                    break;
                }
            }
            None => panic!("Expected terminator, got nothing"),
        }
    }
    output
}

pub fn collect_hex(iterator: &mut Peekable<Chars>) -> u8 {
    // Probably there is a more elegant solution, but this works.
    let first = iterator.next().unwrap();
    let second = iterator.next().unwrap();
    let mut hex = String::new();
    hex.push(first);
    hex.push(second);
    u8::from_str_radix(&hex, 16).unwrap()
}

/// Collects a hex number. The iterator must be placed on the '#' character
pub fn collect_hex_color(iterator: &mut Peekable<Chars>) -> Color {
    assert_eq!(iterator.next(), Some('#'));

    let r = collect_hex(iterator);
    let g = collect_hex(iterator);
    let b = collect_hex(iterator);

    Color::Hex(r, g, b)
}

/// The iterator has to be placed at the first starting character of the CSS value. The iterator
/// will return in the `;` character's position
pub fn parse_css_value(iterator: &mut Peekable<Chars>) -> Vec<Value> {
    let mut output = Vec::new();
    let a = iterator.clone();
    loop {
        match iterator.peek() {
            Some(v) if v.is_alphabetic() || v.is_numeric() || *v == '.' => loop {
                let value = collect_until_terminator(iterator, &[';', ' ']);
                output.push(Value::from_string(&value));
                match iterator.peek() {
                    Some(';') => break,
                    Some(' ') => skip_whitespace(iterator),
                    _ => panic!("Invalid state"),
                }
            },

            Some(';') => break,

            Some('#') => {
                let color = collect_hex_color(iterator);
                output.push(Value::Color(color));
            }

            Some(v) => panic!("Invalid state '{}'\n{:?}", v, a),

            None => panic!("Expected more values"),
        }
    }
    output
}

/// Just a simple function to transform css into a valid block. An inefficient solution, but it
/// works, so ehh...
pub fn parse_inline_css(inline_css: &str) -> Vec<Rule> {
    let blockified_code = "{".to_owned() + inline_css + ";}";
    let mut iter = blockified_code.chars().peekable();
    parse_block(&mut iter)
}

/// Parses a css block
///
/// The iterator has to start a `{` character. The iterator will then be placed to the closing `}`
/// character after this function returns.
pub fn parse_block(iterator: &mut Peekable<Chars>) -> Vec<Rule> {
    assert_eq!(iterator.next(), Some('{'));
    let mut rules = Vec::new();
    loop {
        match iterator.peek() {
            Some('}') => break,

            Some(v) if v.is_whitespace() => {
                skip_whitespace(iterator);
            }

            Some(v) if v.is_alphabetic() => {
                let identifier = get_identifier(iterator);
                println!("Value Identifier - {}", identifier);

                skip_whitespace(iterator);
                assert_eq!(iterator.next(), Some(':'));
                skip_whitespace(iterator);

                let value = parse_css_value(iterator);
                assert_eq!(iterator.next(), Some(';'));

                if let Some(rule) = Rule::new(&identifier, value) {
                    rules.push(rule);
                }
            }

            Some(';') => {
                println!("Extra ';' found");
                iterator.next();
            }

            _ => panic!(
                "Unimplemented CSS syntax character found: '{:?}'",
                iterator.next()
            ),
        }
    }
    rules
}

pub fn parse_css(source: &str) -> Vec<RuleSet> {
    let mut iterator = source.chars().peekable();
    let mut rule_sets = Vec::new();
    loop {
        match iterator.peek() {
            Some(v) if v.is_whitespace() => {
                skip_whitespace(&mut iterator);
            }

            Some(v) => {
                let identifier = get_identifier(&mut iterator);
                let selector = Selector::new(&identifier);
                println!("Identifier: '{}'", identifier);

                skip_whitespace(&mut iterator);
                assert_eq!(iterator.peek(), Some('{').as_ref());

                let rules = parse_block(&mut iterator);
                assert_eq!(iterator.next(), Some('}'));
                println!("Parsed block");

                rule_sets.push(RuleSet::new(vec![selector], rules));
            }

            None => break,

            _ => panic!("Unhandled CSS syntax character '{:?}'", iterator.peek()),
        }
    }
    rule_sets
}
