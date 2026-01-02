#![warn(
	clippy::pedantic,
	clippy::nursery,
	clippy::cargo,
	// Restriction category
	clippy::allow_attributes_without_reason)]

mod cursor;
mod environment;
mod error;
mod evaluator;
mod func;
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
