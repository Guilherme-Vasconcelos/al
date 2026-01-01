use std::error::Error;
use std::fmt;

use crate::object::{LispObject, LispObjectKind};

#[derive(Debug, PartialEq, Eq)]
pub enum PrimitiveError {
	WrongType,
}

impl fmt::Display for PrimitiveError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::WrongType => write!(f, "wrong type"),
		}
	}
}

impl Error for PrimitiveError {}

pub fn add(args: LispObject) -> Result<LispObject, PrimitiveError> {
	if let LispObjectKind::Cons(_) = &args.kind {
		let mut iterator = args.into_iter();
		if iterator
			.filter(|obj| obj.kind == LispObjectKind::Nil)
			.collect::<Vec<_>>()
			.len() > 1
		{
			// There's exactly 1 NIL that is acceptable: the trailing NIL that every list has.
			// If the list contains any man-made NIL, it's an error.
			return Err(PrimitiveError::WrongType);
		}

		let mut iterator = args.into_iter();
		let sum = iterator.try_fold(0, |acc, obj| {
			if let LispObjectKind::Num(n) = obj.kind {
				// TODO: Right now, we use `wrapping_add` to avoid a panic. But in the future
				// it would be good to support big nums.
				Ok(i64::wrapping_add(acc, n))
			} else if let LispObjectKind::Nil = obj.kind {
				Ok(acc)
			} else {
				Err(PrimitiveError::WrongType)
			}
		})?;

		Ok(LispObject::new_num(sum))
	} else {
		panic!("args to the add function must be a cons cell");
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::reader::parse_tokens;
	use crate::tokenizer::parse_bytes;

	fn build_obj_from_src(src: &str) -> LispObject {
		let tokens = parse_bytes(src.as_bytes()).unwrap();
		let objs = parse_tokens(&tokens).unwrap();
		assert!(objs.len() == 1);
		objs[0].clone()
	}

	#[test]
	fn test_sum_of_flat_list() {
		let input = "(1 2 3 4)";
		let obj = build_obj_from_src(input);
		let result = add(obj).unwrap();
		assert_eq!(result, LispObject::new_num(1 + 2 + 3 + 4));
	}

	#[test]
	fn test_list_with_wrong_nil_type() {
		let input = "(1 2 nil 3 4)";
		let obj = build_obj_from_src(input);
		let result = add(obj);
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), PrimitiveError::WrongType);
	}

	#[test]
	fn test_list_with_wrong_list_type() {
		let input = "(1 2 (123) 3 4)";
		let obj = build_obj_from_src(input);
		let result = add(obj);
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), PrimitiveError::WrongType);
	}
}
