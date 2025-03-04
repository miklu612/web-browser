use std::{iter::Peekable, str::Chars};

#[derive(Debug)]
enum Tag {
    PlainText,
    Header(u32),
    Div,
    Body,
    Html,
}

impl Tag {
    pub fn from_string(tag: &str) -> Result<Self, String> {
        match tag {
            "h1" => Ok(Tag::Header(1)),
            "div" => Ok(Tag::Div),
            "body" => Ok(Tag::Body),
            "html" => Ok(Tag::Html),
            v => Err(format!("Unknown tag: {}", v)),
        }
    }
}

#[derive(Debug)]
pub struct Element {
    pub element_type: Tag,
    pub children: Vec<Element>,
    pub inner_text: String,
}

impl Element {
    pub fn new(tag: Tag) -> Self {
        Self {
            element_type: tag,
            children: Vec::new(),
            inner_text: String::new(),
        }
    }

    pub fn new_with_text(tag: Tag, inner_text: &str) -> Self {
        Self {
            element_type: tag,
            inner_text: inner_text.to_string(),
            children: Vec::new(),
        }
    }
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

fn parse_element_content(mut iter: &mut Peekable<Chars>, parent_tag: &str) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();
    while let Some(character) = iter.peek() {
        match character {
            '<' => match iter.clone().nth(1) {
                Some('/') => {
                    break;
                }
                Some(v) if v.is_alphabetic() => {
                    let child_element = parse_html_element(&mut iter);
                    elements.push(child_element);
                }
                Some(v) => panic!("Unknown character after '<' : {}", v),
                None => panic!("Expected character after '<'"),
            },
            c => {
                if let Some(mut v) = elements.last_mut() {
                    v.inner_text.push(*c);
                } else {
                    elements.push(Element::new_with_text(Tag::PlainText, &c.to_string()));
                }
                iter.next();
            }
        }
    }

    elements
}

fn parse_html_element(mut iter: &mut Peekable<Chars>) -> Element {
    assert!(
        iter.next_if_eq(&'<').is_some(),
        "Expected 'Some('<')' Got: '{:?}'",
        iter
    );
    let tag = get_identifier(&mut iter);
    assert!(
        iter.next_if_eq(&'>').is_some(),
        "Expected 'Some('>')' Got: '{:?}'",
        iter
    );
    let children = parse_element_content(&mut iter, &tag);
    assert!(
        iter.next_if_eq(&'<').is_some(),
        "Expected 'Some(<)' Got: '{:?}'",
        iter.next()
    );
    assert!(
        iter.next_if_eq(&'/').is_some(),
        "Expected 'Some(/)' Got: '{:?}'",
        iter.next()
    );
    let closing_tag = get_identifier(&mut iter);
    assert!(iter.next() == Some('>'));
    assert!(tag == closing_tag);

    let mut element = Element::new(Tag::from_string(&tag).unwrap());
    element.children = children;
    element
}

fn parse_html_iter(mut iter: &mut Peekable<Chars>) -> Vec<Element> {
    let mut elements = Vec::new();

    loop {
        match iter.peek() {
            Some('<') => {
                let element = parse_html_element(iter);
                elements.push(element);
            }
            Some(v) if v.is_whitespace() => {
                iter.next();
            }
            Some(v) => panic!("Unhandled character in HTML: '{}'", v),
            None => break,
        }
    }

    elements
}

pub fn parse_html(html: &str) -> Vec<Element> {
    parse_html_iter(&mut html.chars().peekable())
}
