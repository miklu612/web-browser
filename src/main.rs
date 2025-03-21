use font::Font;
use html::parse_html;
use requests::get_site;
use std::{fs::File, io::prelude::Read, path::Path};
use window::Window;

mod bound;
mod css;
mod document;
mod font;
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

fn from_file(path: &Path) {
    let website_code = read_file(path);
    let elements = parse_html(&website_code);
    let mut window = Window::new();
    window.render(elements);
}

fn from_web(path: &str) {
    let website_code = get_site(path);
    let elements = parse_html(&website_code);
    let mut window = Window::new();
    window.render(elements);
}

fn render_text(text: &str) {
    let font = Font::load(Path::new(
        "./fonts/liberation-sans/LiberationSans-Regular.ttf",
    ))
    .unwrap();
    let image = font.render_string(text);
    image.save("output.png").unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 {
        if args[1] == "--from-file" {
            from_file(Path::new(&args[2]));
        } else if args[1] == "--from-web" {
            from_web(&args[2]);
        } else if args[1] == "--render-text" {
            render_text(&args[2]);
        } else {
            panic!("Unknown argument");
        }
    } else {
        from_file(Path::new("./tests/index.html"));
    }
}
