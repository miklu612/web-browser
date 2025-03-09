use html::{parse_html, Tag};
use std::{fs::File, io::prelude::Read, path::Path};
use window::Window;

mod html;
mod window;

fn read_file(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    content
}

fn main() {
    let mut args = std::env::args();

    let file_name;
    if args.len() == 2 {
        file_name = args.nth(1).unwrap();
    } else {
        file_name = "./tests/index.html".to_owned();
    }

    let elements = parse_html(&read_file(&Path::new(&file_name)));
    assert!(elements.len() == 1);
    assert!(elements[0].element_type == Tag::Html);
    assert!(elements[0].children.len() == 1);
    assert!(elements[0].children[0].element_type == Tag::Body);

    let mut window = Window::new();
    window.render(elements);
}
