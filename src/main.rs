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
    let mut window = Window::new();
    window.open();
}
