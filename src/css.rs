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
#[derive(Debug)]
pub enum Unit {
    Px(i32),
}

/// This won't be used in the final product, but it will be used to contain a variable value to
/// return in function calls. It will be up to the function implementation to validate that this
/// value contains the correct value.
pub enum Value {
    Unit(Unit),
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
}

impl Rule {
    pub fn new(identifier: &str, value: Value) -> Self {
        match identifier {
            "width" => match value {
                Value::Unit(unit) => Self::Width(unit),
                _ => panic!("Expected unit"),
            },

            _ => panic!("Unknown css identifier: {}", identifier),
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
    let mut output = None;
    loop {
        match iterator.peek() {
            Some(v) if v.is_numeric() => {
                digit = digit * 10 + v.to_digit(10).unwrap() as i32;
                iterator.next();
            }

            Some('p') => {
                assert_eq!(iterator.next(), Some('p'));
                assert_eq!(iterator.next(), Some('x'));
                assert_eq!(iterator.peek(), Some(';').as_ref());
                output = Some(Unit::Px(digit));
                break;
            }

            _ => panic!("Unhandled character in value unit: '{:?}'", iterator.next()),
        }
    }
    output.expect("Couldn't evaluate value unit type")
}

/// The iterator has to be placed at the first starting character of the CSS value. The iterator
/// will return in the `;` character's position
pub fn parse_css_value(iterator: &mut Peekable<Chars>) -> Value {
    match iterator.peek() {
        Some(v) if v.is_numeric() => Value::Unit(parse_css_value_unit(iterator)),

        _ => panic!(
            "Unimplemented CSS syntax character found: '{:?}'",
            iterator.next()
        ),
    }
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

            Some(v) => {
                let identifier = get_identifier(iterator);
                println!("Value Identifier - {}", identifier);

                skip_whitespace(iterator);
                assert_eq!(iterator.next(), Some(':'));
                skip_whitespace(iterator);

                let value = parse_css_value(iterator);
                assert_eq!(iterator.next(), Some(';'));

                rules.push(Rule::new(&identifier, value));
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
