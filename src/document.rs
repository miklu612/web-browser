use crate::css::RuleSet;
use crate::html::Element;

#[allow(dead_code)]
pub struct Document {
    pub elements: Vec<Element>,
    pub css_rules: Vec<RuleSet>,
}

impl Document {
    pub fn new(elements: Vec<Element>, css_rules: Vec<RuleSet>) -> Self {
        Self {
            elements,
            css_rules,
        }
    }

    pub fn parse_inline_css(&mut self) {
        for element in &mut self.elements {
            element.parse_inline_css();
        }
    }
}
