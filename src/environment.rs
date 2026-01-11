use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::object::LispObject;
use crate::prim;

pub type Env = Rc<RefCell<Environment>>;

pub fn new_env(parent: Env) -> Env {
	Rc::new(RefCell::new(Environment {
		parent: Some(parent),
		values: HashMap::new(),
	}))
}

pub fn new_global_env() -> Env {
	let mut env = Environment {
		parent: None,
		values: HashMap::new(),
	};
	env.set("+", LispObject::new_primitive(prim::primitive_add));
	Rc::new(RefCell::new(env))
}

/// Do not instantiate an Environment directly. Creating an Environment without an `Rc<RefCell<T>>` container will
/// cause issues. Instead, rely on `new_env` and `new_global_env`, both of which return an `Rc<RefCell<Environment>>`.
#[derive(Debug, PartialEq, Eq)]
pub struct Environment {
	parent: Option<Env>,
	values: HashMap<String, LispObject>,
}

impl Environment {
	pub fn set(&mut self, key: &str, value: LispObject) {
		self.values.insert(key.to_owned(), value);
	}

	pub fn get(&self, key: &str) -> Option<LispObject> {
		self.values.get(key).cloned()
	}

	pub const fn is_global(&self) -> bool {
		self.parent.is_none()
	}

	pub fn hierarchical_get(&self, key: &str) -> Option<LispObject> {
		self.get(key).or_else(|| {
			self.parent
				.as_ref()
				.and_then(|p| p.borrow().hierarchical_get(key))
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::environment;
	use crate::environment::new_global_env;
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
		let env = new_global_env();
		let mut env = env.borrow_mut();
		env.set("hello", LispObject::new_num(2));
		assert_eq!(env.get("hello").unwrap(), LispObject::new_num(2));
	}

	#[test]
	fn test_insert_existing_item() {
		let env = new_global_env();
		let mut env = env.borrow_mut();
		env.set("hello", LispObject::new_num(2));
		env.set("hello", LispObject::new_num(77));
		assert_eq!(env.get("hello").unwrap(), LispObject::new_num(77));
	}

	#[test]
	fn test_an_env_without_parent_is_global() {
		let env = new_global_env();
		let env_b = env.borrow();
		assert!(env_b.is_global());
		let env2 = environment::new_env(env.clone());
		let env2_b = env2.borrow();
		assert!(!env2_b.is_global());
	}

	#[test]
	fn test_call_func_through_env() {
		let env = new_global_env();
		let env = env.borrow();
		let primitive_add = env.get("+");
		let input = "(1 2 3)";
		let args = build_obj_from_src(input);
		let result = primitive_add.unwrap().call_func(args);
		assert_eq!(result, Ok(LispObject::new_num(1 + 2 + 3)));
	}
}
