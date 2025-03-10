use std::{collections::HashMap, iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tag {
    PlainText,
    H(u32),
    Div,
    Body,
    Html,
    Paragraph,
    Title,
    Meta,
    Link,
    Head,
    Span,
    Center,
    Input,
    Table,
    Script,
    Header,
    Form,
    Img,
    Li,
    Td,
    Ul,
    Br,
    Tr,
    A,
    B,
    U,
}

impl Tag {
    pub fn from_string(tag: &str) -> Result<Self, String> {
        match tag {
            "h1" => Ok(Tag::H(1)),
            "div" => Ok(Tag::Div),
            "body" => Ok(Tag::Body),
            "html" => Ok(Tag::Html),
            "p" => Ok(Tag::Paragraph),
            "title" => Ok(Tag::Title),
            "header" => Ok(Tag::Header),
            "meta" => Ok(Tag::Meta),
            "head" => Ok(Tag::Head),
            "link" => Ok(Tag::Link),
            "img" => Ok(Tag::Img),
            "a" => Ok(Tag::A),
            "b" => Ok(Tag::B),
            "u" => Ok(Tag::U),
            "td" => Ok(Tag::Td),
            "ul" => Ok(Tag::Ul),
            "tr" => Ok(Tag::Tr),
            "li" => Ok(Tag::Li),
            "br" => Ok(Tag::Br),
            "table" => Ok(Tag::Table),
            "center" => Ok(Tag::Center),
            "span" => Ok(Tag::Span),
            "input" => Ok(Tag::Input),
            "form" => Ok(Tag::Form),
            "script" => Ok(Tag::Script),
            v => Err(format!("Unknown tag: {}", v)),
        }
    }
}

#[derive(Debug)]
pub struct Element {
    pub element_type: Tag,
    pub children: Vec<Element>,
    pub inner_text: String,
    pub attributes: HashMap<String, String>,
}

impl Element {
    pub fn new(tag: Tag) -> Self {
        Self {
            element_type: tag,
            children: Vec::new(),
            inner_text: String::new(),
            attributes: HashMap::new(),
        }
    }

    pub fn new_with_text(tag: Tag, inner_text: &str) -> Self {
        Self {
            element_type: tag,
            inner_text: inner_text.to_string(),
            children: Vec::new(),
            attributes: HashMap::new(),
        }
    }
}

pub fn get_identifier(iter: &mut Peekable<Chars>) -> String {
    let mut string = String::new();
    while let Some(character) = iter.peek() {
        if character.is_alphabetic()
            || (character.is_numeric() && !string.is_empty())
            || *character == '-'
        {
            string.push(*character);
            iter.next();
        } else {
            break;
        }
    }
    string
}

/// Gets text content of an element when the iterator is set inside of it.
#[allow(dead_code)]
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

fn parse_element_content(iter: &mut Peekable<Chars>) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();
    while let Some(character) = iter.peek() {
        match character {
            '<' => match iter.clone().nth(1) {
                Some('/') => {
                    break;
                }
                Some(v) if v.is_alphabetic() => {
                    let child_element = parse_html_element(iter);
                    elements.push(child_element);
                }
                Some(v) => panic!("Unknown character after '<' : {}", v),
                None => panic!("Expected character after '<'"),
            },
            c => {
                if let Some(v) = elements.last_mut() {
                    v.inner_text.push(*c);
                } else {
                    elements.push(Element::new_with_text(Tag::PlainText, &c.to_string()));
                }
                iter.next();
            }
        }
    }

    // Strip all of the inner text content
    elements
        .iter_mut()
        .for_each(|x| x.inner_text = x.inner_text.trim().to_owned().replace("&nbsp;", " "));

    // Filter out empty PlainText elements
    elements.retain(|x| {
        if x.element_type == Tag::PlainText {
            !x.inner_text.is_empty()
        } else {
            true
        }
    });

    elements
}

fn is_quotation(character: char) -> bool {
    character == '\'' || character == '"'
}

/// Gets a string without the quotation marks
fn get_string(iter: &mut Peekable<Chars>) -> String {
    assert!(is_quotation(iter.next().unwrap()));
    let mut output = String::new();
    loop {
        match iter.next() {
            Some(v) if is_quotation(v) => break,
            Some(v) => output.push(v),
            None => panic!("String ran out"),
        }
    }
    output
}

fn parse_attributes(iter: &mut Peekable<Chars>) -> HashMap<String, String> {
    let mut output = HashMap::<String, String>::new();
    loop {
        match iter.peek() {
            Some('>') => break,
            Some('/') => break,
            Some(v) if v.is_whitespace() => {
                iter.next();
            }
            Some(_) => {
                let identifier = get_identifier(iter);
                assert_eq!(iter.next(), Some('='));
                let string = get_string(iter);
                output.insert(identifier, string);
            }
            None => panic!("Attributes ran out"),
        }
    }
    output
}

fn parse_html_element(iter: &mut Peekable<Chars>) -> Element {
    assert!(
        iter.next_if_eq(&'<').is_some(),
        "Expected 'Some('<')' Got: '{:?}'",
        iter
    );
    let tag = get_identifier(iter);
    let attributes = parse_attributes(iter);

    // These elements for whatever reason aren't self terminating sometimes, so we gotta check for them
    if tag == "link" || tag == "meta" || tag == "img" || tag == "input" || tag == "br" {
        assert_eq!(iter.next(), Some('>'));
        let mut element = Element::new(Tag::from_string(&tag).unwrap());
        element.attributes = attributes;
        return element;
    }

    match iter.peek() {
        Some('/') => {
            iter.next();
            assert_eq!(iter.next(), Some('>'));
            let mut element = Element::new(Tag::from_string(&tag).unwrap());
            element.attributes = attributes;
            return element;
        }
        Some('>') => {
            iter.next();
        }
        _ => panic!("Expected more after attributes"),
    }

    let children = parse_element_content(iter);
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
    let closing_tag = get_identifier(iter);
    assert_eq!(iter.next(), Some('>'));
    assert_eq!(tag, closing_tag);

    let mut element = Element::new(Tag::from_string(&tag).unwrap());
    element.children = children;
    element.attributes = attributes;
    element
}

fn parse_html_iter(iter: &mut Peekable<Chars>) -> Vec<Element> {
    let mut elements = Vec::new();

    loop {
        match iter.peek() {
            Some('<') => {
                if iter.clone().nth(1) == Some('!') {
                    let doctype_string = "<!DOCTYPE html>".to_lowercase();
                    for character in doctype_string.chars() {
                        assert_eq!(
                            iter.next().unwrap().to_lowercase().next().unwrap(),
                            character
                        );
                    }
                } else {
                    let element = parse_html_element(iter);
                    elements.push(element);
                }
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
