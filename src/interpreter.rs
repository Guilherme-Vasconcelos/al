use std::error::Error;
use std::fs::File;
use std::io::Read;

use crate::reader::parse_tokens;
use crate::tokenizer::parse_bytes;

pub fn run_from_file(path: &str) -> Result<(), Box<dyn Error>> {
	let mut file = File::open(path)?;
	let mut contents = Vec::new();
	file.read_to_end(&mut contents)?;

	let tokens = parse_bytes(&contents)?;
	let objs = parse_tokens(&tokens)?;
	dbg!(objs);

	Ok(())
}
