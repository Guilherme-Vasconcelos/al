use std::rc::Rc;

use crate::environment::Environment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LispObjectKind {
	Num(i64),
	Sym(Rc<String>),
	r#Cons(Cons),
	r#Func(Func),
	Nil,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LispObject {
	pub kind: LispObjectKind,
}

impl LispObject {
	pub const fn new_nil() -> Self {
		Self {
			kind: LispObjectKind::Nil,
		}
	}

	pub const fn new_num(num: i64) -> Self {
		Self {
			kind: LispObjectKind::Num(num),
		}
	}

	pub fn new_sym(sym: String) -> Self {
		Self {
			kind: LispObjectKind::Sym(sym.into()),
		}
	}

	pub fn new_cons(car: Self, cdr: Self) -> Self {
		Self {
			kind: LispObjectKind::Cons(Cons::new(car, cdr)),
		}
	}

	pub fn new_empty_cons() -> Self {
		Self::new_cons(Self::new_nil(), Self::new_nil())
	}
}

impl<'a> IntoIterator for &'a LispObject {
	type Item = &'a LispObject;
	type IntoIter = LispObjectIter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		LispObjectIter {
			current: Some(self),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cons {
	pub car: Box<LispObject>,
	pub cdr: Box<LispObject>,
}

impl Cons {
	pub fn new(car: LispObject, cdr: LispObject) -> Self {
		Self {
			car: Box::new(car),
			cdr: Box::new(cdr),
		}
	}

	pub fn empty() -> Self {
		Self::new(LispObject::new_nil(), LispObject::new_nil())
	}
}

#[allow(
	unpredictable_function_pointer_comparisons,
	reason = "We need Func to be PartialEq+Eq so that LispObject can be PartialEq+Eq, but we do not compare functions."
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Func {
	/// Args -> Return
	Primitive(fn(LispObject) -> LispObject),
	/// Body, Environment, Args -> Return
	Closure(fn(LispObject, Environment, LispObject) -> LispObject),
}

/// This is a helper that allows you to easily iterate over a `LispObject`.
/// Notice that iterating over a `LispObject` is most meaningful when the object is
/// a cons cell--otherwise, it just yields self and then starts always yielding None.
pub struct LispObjectIter<'a> {
	current: Option<&'a LispObject>,
}

impl<'a> Iterator for LispObjectIter<'a> {
	type Item = &'a LispObject;

	fn next(&mut self) -> Option<Self::Item> {
		let obj = self.current?;
		if let LispObjectKind::Cons(cons) = &obj.kind {
			self.current = Some(&cons.cdr);
			Some(&cons.car)
		} else {
			// Yield the last object, but nothing more will be yielded after this.
			self.current = None;
			Some(obj)
		}
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
	fn create_iterator_for_flat_list() {
		let input = "(1 2 3 4)";
		let obj = build_obj_from_src(input);
		let mut iterator = obj.into_iter();
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_num(1));
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_num(2));
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_num(3));
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_num(4));
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_nil());
		assert!(iterator.next().is_none());
	}

	#[test]
	fn create_iterator_for_list_with_sublists() {
		let input = "(1 (2) 3)";
		let obj = build_obj_from_src(input);
		let mut iterator = obj.into_iter();
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_num(1));
		assert_eq!(
			*(iterator.next().unwrap()),
			LispObject::new_cons(LispObject::new_num(2), LispObject::new_nil())
		);
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_num(3));
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_nil());
		assert!(iterator.next().is_none());
	}

	#[test]
	fn create_iterator_for_atom() {
		let input = "55";
		let obj = build_obj_from_src(input);
		let mut iterator = obj.into_iter();
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_num(55));
		assert!(iterator.next().is_none());
	}
}
