//! Print just the rendered body (what window.rerender receives on live reload).
use picomd::render;
fn main() {
    let path = std::env::args().nth(1).expect("usage: body <file.md>");
    let md = std::fs::read_to_string(&path).expect("read");
    print!("{}", render(&md));
}
