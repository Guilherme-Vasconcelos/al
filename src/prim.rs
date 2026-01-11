use crate::object::{FuncallError, LispObject, LispObjectKind};

#[allow(
	clippy::needless_pass_by_value,
	reason = "The signature must be the same for every primitive so that it can be stored as a function pointer. However, there's no guarantee that every primitive will not consume the object. May be reviewed in the future when we have more examples."
)]
pub fn primitive_add(args: LispObject) -> Result<LispObject, FuncallError> {
	if let LispObjectKind::Cons(_) = &args.kind {
		let mut iterator = args.into_iter();
		let sum = iterator.try_fold(0, |acc, obj| {
			if let LispObjectKind::Num(n) = obj.kind {
				// TODO: Right now, we use `wrapping_add` to avoid a panic. But in the future
				// it would be good to support big nums.
				Ok(i64::wrapping_add(acc, n))
			} else {
				Err(FuncallError::WrongType)
			}
		})?;

		if iterator.is_proper() {
			Ok(LispObject::new_num(sum))
		} else {
			Err(FuncallError::WrongType)
		}
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
		let result = primitive_add(obj).unwrap();
		assert_eq!(result, LispObject::new_num(1 + 2 + 3 + 4));
	}

	#[test]
	fn test_list_with_wrong_nil_type() {
		let input = "(1 2 nil 3 4)";
		let obj = build_obj_from_src(input);
		let result = primitive_add(obj);
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), FuncallError::WrongType);
	}

	#[test]
	fn test_list_with_wrong_list_type() {
		let input = "(1 2 (123) 3 4)";
		let obj = build_obj_from_src(input);
		let result = primitive_add(obj);
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), FuncallError::WrongType);
	}

	#[test]
	fn test_improper_list() {
		let obj = LispObject::new_cons(
			LispObject::new_sym("+".into()),
			LispObject::new_cons(LispObject::new_num(1), LispObject::new_num(2)),
		);
		let result = primitive_add(obj);
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), FuncallError::WrongType);
	}
}
