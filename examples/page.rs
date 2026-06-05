//! Print the full HTML page for a Markdown file, the same output the `/` route
//! serves.
//!   cargo run --example page -- input.md > out.html
use picomd::{render, template};

fn main() {
    let path = std::env::args().nth(1).expect("usage: page <file.md>");
    let md = std::fs::read_to_string(&path).expect("read input");
    print!("{}", template::page(&render(&md)));
}
