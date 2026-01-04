use std::error::Error;
use std::fs::File;
use std::io::Read;

use crate::environment::new_global_env;
use crate::evaluator::eval;
use crate::reader::parse_tokens;
use crate::tokenizer::parse_bytes;

pub fn run_from_file(path: &str) -> Result<(), Box<dyn Error>> {
	let mut file = File::open(path)?;
	let mut contents = Vec::new();
	file.read_to_end(&mut contents)?;

	let tokens = parse_bytes(&contents)?;
	let objs = parse_tokens(&tokens)?;
	let global_env = new_global_env();
	for obj in objs {
		let base = obj.clone();
		let res = eval(global_env.clone(), &obj);
		if let Ok(ores) = res {
			println!("------------\nEVALUATING: {base}\nRESULT: {ores}\n------------\n");
		} else {
			println!(
				"------------\nEVALUATING: {}\nRESULT: {}\n------------\n",
				base,
				res.unwrap_err()
			);
		}
	}

	Ok(())
}
