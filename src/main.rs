use css::parse_css;
use html::{parse_html, Tag};
use requests::get_site;
use std::{fs::File, io::prelude::Read, path::Path};
use window::Window;

mod css;
mod document;
mod html;
mod render_layout;
mod requests;
mod window;

fn read_file(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    content
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut as_css = false;
    let mut website_code = if args.len() == 3 {
        if args[1] == "--from-file".to_owned() {
            read_file(Path::new(&args[2]))
        } else if args[1] == "--from-web" {
            get_site(&args[2])
        } else if args[1] == "--from-css-file" {
            as_css = true;
            read_file(Path::new(&args[2]))
        } else {
            panic!("Unknown argument")
        }
    } else {
        read_file(Path::new("./tests/index.html"))
    };

    if !as_css {
        let elements = parse_html(&website_code);
        println!("{:?}", elements);
        assert!(elements.len() == 1);
        assert!(elements[0].element_type == Tag::Html);

        let mut window = Window::new();
        window.render(elements);
    } else {
        let css_rules = parse_css(&website_code);
        println!("Parsed css:\n{:?}", css_rules);
    }
}
