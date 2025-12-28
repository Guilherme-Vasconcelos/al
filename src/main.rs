#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

mod cursor;
mod environment;
mod error;
mod interpreter;
mod object;
mod reader;
mod tokenizer;

use crate::error::die;

fn main() {
	let mut args = std::env::args();
	if args.len() != 2 {
		die("Usage: al <file>");
	}

	let fpath = &args.nth(1).unwrap();
	if let Err(e) = interpreter::run_from_file(fpath) {
		// When possible, errors should be handled before they reach main.
		// But if they aren't, by reaching here it means we should terminate.
		die(e);
	}
}
