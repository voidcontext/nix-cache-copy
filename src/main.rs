use std::io;

use crate::parser::Line;

mod parser;

fn main() {
    let stdin = io::stdin();

    for line in stdin.lines() {
        let line = Line::parse(line.unwrap());
        println!("{line:?}");
    }
}
