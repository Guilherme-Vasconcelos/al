#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

mod error;
mod interpreter;
mod tokenizer;

use crate::error::die;

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        die("Usage: al <file>");
    }

    let fpath = &args.nth(1).unwrap();
    if let Err(e) = interpreter::run_from_file(fpath) {
        die(e);
    }
}
