use std::collections::HashMap;

use crate::func;
use crate::object::LispObject;

#[derive(Debug)]
pub struct Environment<'env, 'key> {
	parent: Option<&'env Self>,
	values: HashMap<&'key str, LispObject>,
}

impl<'env, 'key> Environment<'env, 'key> {
	pub fn new(parent: &'env Self) -> Self {
		Self {
			parent: Some(parent),
			values: HashMap::new(),
		}
	}

	pub fn new_global() -> Self {
		let mut env = Self {
			parent: None,
			values: HashMap::new(),
		};
		env.set("+", LispObject::new_primitive(func::primitive_add));
		env
	}

	pub fn set(&mut self, key: &'key str, value: LispObject) {
		let existing = self.get_mut(key);
		if let Some(e) = existing {
			*e = value;
		} else {
			self.values.insert(key, value);
		}
	}

	pub fn get(&self, key: &'key str) -> Option<&LispObject> {
		self.values.get(key)
	}

	fn get_mut(&mut self, key: &'key str) -> Option<&mut LispObject> {
		self.values.get_mut(key)
	}

	pub const fn is_global(&self) -> bool {
		self.parent.is_none()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::object::LispObject;
	use crate::reader::parse_tokens;
	use crate::tokenizer::parse_bytes;

	fn build_obj_from_src(src: &str) -> LispObject {
		let tokens = parse_bytes(src.as_bytes()).unwrap();
		let objs = parse_tokens(&tokens).unwrap();
		assert!(objs.len() == 1);
		objs[0].clone()
	}

	#[test]
	fn test_insert_and_fetch_atom() {
		let mut env = Environment::new_global();
		env.set("hello", LispObject::new_num(2));
		assert_eq!(*env.get("hello").unwrap(), LispObject::new_num(2));
	}

	#[test]
	fn test_insert_existing_item() {
		let mut env = Environment::new_global();
		env.set("hello", LispObject::new_num(2));
		env.set("hello", LispObject::new_num(77));
		assert_eq!(*env.get("hello").unwrap(), LispObject::new_num(77));
	}

	#[test]
	fn test_an_env_without_parent_is_global() {
		let env = Environment::new_global();
		assert!(env.is_global());
		let env2 = Environment::new(&env);
		assert!(!env2.is_global());
	}

	#[test]
	fn test_call_func_through_env() {
		let env = Environment::new_global();
		let primitive_add = env.get("+");
		let input = "(1 2 3)";
		let args = build_obj_from_src(input);
		let result = primitive_add.unwrap().call_func(args);
		assert_eq!(result, Ok(LispObject::new_num(1 + 2 + 3)));
	}
}
