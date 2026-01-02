use std::fmt;
use std::rc::Rc;

use crate::environment::Environment;
use crate::func::FuncallError;

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

	pub fn new_primitive(func: fn(Self) -> Result<Self, FuncallError>) -> Self {
		Self {
			kind: LispObjectKind::Func(Func::Primitive(func)),
		}
	}

	/// Panics if `self` is not a function.
	pub fn call_func(&self, args: Self) -> Result<Self, FuncallError> {
		match &self.kind {
			LispObjectKind::Func(f) => match f {
				Func::Primitive(p) => p(args),
				Func::Closure(_c) => {
					panic!("closure is not yet supported")
				}
			},
			_ => panic!("unable to call non-function as a function"),
		}
	}
}

impl fmt::Display for LispObject {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		match &self.kind {
			LispObjectKind::Num(n) => {
				write!(f, "ATOM(type=NUM, value={n})")
			}
			LispObjectKind::Sym(s) => {
				write!(f, "ATOM(type=SYM, value='{s}')")
			}
			LispObjectKind::Cons(c) => {
				write!(f, "CONS(car={}, cdr={})", c.car, c.cdr)
			}
			LispObjectKind::Nil => {
				write!(f, "NIL")
			}
			LispObjectKind::Func(func) => match func {
				// TODO: Ideally show something about the body/args/env.
				Func::Primitive(_) => {
					write!(f, "NATIVE FUNCTION")
				}
				Func::Closure(_) => {
					write!(f, "CLOSURE")
				}
			},
		}
	}
}

impl<'a> IntoIterator for &'a LispObject {
	type Item = &'a LispObject;
	type IntoIter = LispObjectIter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		LispObjectIter::new(self)
	}
}

/// A helper to build cons cells without having to deal with manually setting
/// CAR and CDR for each element.
/// For example, the following two are equivalent:
///
/// ```rust
/// let obj = LispObject::new_cons(LispObject::new_num(1), LispObject::new_nil());
/// let mut builder = LispObjConsBuilder::new();
/// builder.push(LispObject::new_num(1));
/// let obj2 = builder.build();
/// assert_eq!(obj, obj2);
/// ```
///
/// Notice the builder only builds proper lists. You do not have to manually set
/// its CDR to NIL.
pub struct LispObjConsBuilder {
	root: LispObject,
	is_new: bool,
}

impl LispObjConsBuilder {
	pub fn new() -> Self {
		let root = LispObject::new_empty_cons();
		LispObjConsBuilder { root, is_new: true }
	}

	#[cfg(test)]
	pub fn from(objs: Vec<LispObject>) -> Self {
		let mut builder = LispObjConsBuilder::new();
		for obj in objs {
			builder.push(obj);
		}
		builder
	}

	pub fn push(&mut self, obj: LispObject) {
		if self.is_new {
			self.push_first_element(obj);
		} else {
			self.push_by_transversing(obj);
		}
	}

	fn push_first_element(&mut self, obj: LispObject) {
		assert!(self.is_new);
		self.is_new = false;
		if let LispObjectKind::Cons(c) = &mut self.root.kind {
			*c.car = obj;
		} else {
			panic!("root is not a Cons");
		}
	}

	fn push_by_transversing(&mut self, obj: LispObject) {
		let mut current = &mut self.root;

		loop {
			match &mut current.kind {
				LispObjectKind::Cons(cons) => {
					if matches!(cons.cdr.kind, LispObjectKind::Nil) {
						*cons.cdr = LispObject::new_cons(obj, LispObject::new_nil());
						return;
					} else {
						current = &mut cons.cdr;
					}
				}
				_ => panic!("root is not a Cons"),
			}
		}
	}

	pub fn build(self) -> LispObject {
		self.root
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
	Primitive(fn(LispObject) -> Result<LispObject, FuncallError>),
	/// Body, Environment, Args -> Return
	/// FIXME: I shouldn't pass the environment when *calling* the function, therefore it should live
	/// in the Func object, not in the function pointer.
	Closure(fn(LispObject, Environment, LispObject) -> LispObject),
}

/// This is an iterator to help traverse CARs of lists.
/// Notice that the tail of a list is never yielded. Use `iterator.tail` instead.
/// For example:
/// - (1 2 3) -> yields 1, then 2, then 3.
/// - (1 2 . 3) -> yields 1, then 2. If you want the 3, use `iterator.tail`.
/// `tail` is only set once the iterator is consumed. If you try to access it earlier,
/// it will always yield None (even if the list is improper); and if you try to call
/// `self.is_proper()` you'll always get a panic.
pub struct LispObjectIter<'a> {
	current: Option<&'a LispObject>,
	pub tail: Option<&'a LispObject>,
}

impl<'a> LispObjectIter<'a> {
	pub fn new(start: &'a LispObject) -> Self {
		Self {
			current: Some(start),
			tail: None,
		}
	}

	pub fn is_proper(&self) -> bool {
		assert!(self.tail.is_some(), "iteration has not yet finished");
		self.tail.unwrap().kind == LispObjectKind::Nil
	}
}

impl<'a> Iterator for LispObjectIter<'a> {
	type Item = &'a LispObject;

	fn next(&mut self) -> Option<Self::Item> {
		let obj = self.current?;
		if let LispObjectKind::Cons(cons) = &obj.kind {
			self.current = Some(&cons.cdr);
			Some(&cons.car)
		} else {
			self.tail = Some(obj);
			self.current = None;
			None
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
		assert!(iterator.next().is_none());
		assert_eq!(*(iterator.tail.unwrap()), LispObject::new_nil());
		assert!(iterator.is_proper());
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
		assert!(iterator.next().is_none());
		assert_eq!(*(iterator.tail.unwrap()), LispObject::new_nil());
		assert!(iterator.is_proper());
	}

	#[test]
	fn create_iterator_for_atom() {
		let input = "55";
		let obj = build_obj_from_src(input);
		let mut iterator = obj.into_iter();
		assert!(iterator.next().is_none());
	}

	#[test]
	fn create_iterator_for_improper_list() {
		let list = LispObject::new_cons(LispObject::new_num(1), LispObject::new_num(55));
		let mut iterator = list.into_iter();
		assert_eq!(*(iterator.next().unwrap()), LispObject::new_num(1));
		assert!(iterator.next().is_none());
		assert_eq!(*(iterator.tail.unwrap()), LispObject::new_num(55));
		assert!(!iterator.is_proper());
	}

	#[test]
	fn builder_build_list_of_nums() {
		let mut builder = LispObjConsBuilder::new();
		builder.push(LispObject::new_num(1));
		builder.push(LispObject::new_num(52));
		let result = builder.build();
		assert_eq!(
			result,
			LispObject::new_cons(
				LispObject::new_num(1),
				LispObject::new_cons(LispObject::new_num(52), LispObject::new_nil())
			)
		);
	}

	#[test]
	fn builder_build_list_with_sublists() {
		let mut builder = LispObjConsBuilder::new();
		builder.push(LispObject::new_num(1));
		builder.push(LispObject::new_num(52));
		let inner_result = builder.build();
		let mut builder = LispObjConsBuilder::new();
		builder.push(inner_result.clone());
		let result = builder.build();

		assert_eq!(
			result,
			LispObject::new_cons(inner_result, LispObject::new_nil(),)
		);
	}
}
