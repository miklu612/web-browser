mod html;

fn main() {
    let html_source = "<div>\n\t<h1> Hello, World! </h1>\n</div>";
    println!("Parsing:\n{}", html_source);
    let tags = html::parse_html(html_source);
    println!("{:?}", tags);
}
