use std::{iter::Peekable, str::Chars};

enum ElementType {
    PlainText,
    Header(u32),
}

pub struct Element {
    pub element_type: ElementType,
    pub children: Vec<Element>,
}

pub fn get_identifier(iter: &mut Peekable<Chars>) -> String {
    let mut string = String::new();
    while let Some(character) = iter.peek() {
        if character.is_alphabetic() || (character.is_numeric() && !string.is_empty()) {
            string.push(*character);
            iter.next();
        } else {
            break;
        }
    }
    string
}

pub fn get_text(iter: &mut Peekable<Chars>) -> String {
    let mut string = String::new();
    while let Some(character) = iter.peek() {
        if character.is_alphabetic() || character.is_numeric() || character.is_whitespace() {
            string.push(*character);
            iter.next();
        } else {
            break;
        }
    }
    string
}

fn parse_html_iter(mut iter: Peekable<Chars>) -> Vec<Element> {
    let elements = Vec::new();

    loop {
        match iter.next() {
            Some('<') => {
                let identifier = get_identifier(&mut iter);
                assert!(iter.next() == Some('>'));
                let value = get_text(&mut iter);
                assert!(iter.next() == Some('<'));
                assert!(iter.next() == Some('/'));
                let closing_identifier = get_identifier(&mut iter);
                assert!(iter.next() == Some('>'));
                println!("Found element: {}\nValue: {}", identifier, value);
            }
            Some(v) => panic!("Unhandled character in HTML: '{}'", v),
            None => break,
        }
    }

    elements
}

pub fn parse_html(html: &str) -> Vec<Element> {
    parse_html_iter(html.chars().peekable())
}
