use std::{fs::File, io::prelude::Read, path::Path};

mod html;

fn read_file(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    content
}

fn main() {
    let html_code = read_file(Path::new("./tests/index.html"));
    println!("Parsing the following html code\n{}", html_code);
    let elements = html::parse_html(&html_code);
    println!("Output: {:?}", elements)
}
