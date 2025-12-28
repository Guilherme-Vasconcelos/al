use std::rc::Rc;

use crate::environment::Environment;

#[derive(Debug, PartialEq, Eq)]
pub enum LispObjectKind {
	Num(i64),
	Sym(Rc<String>),
	r#Cons(Cons),
	r#Func(Func),
	Nil,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LispObject {
	pub kind: LispObjectKind,
}

impl LispObject {
	pub fn new(kind: LispObjectKind) -> Self {
		Self { kind }
	}

	pub fn new_nil() -> Self {
		Self {
			kind: LispObjectKind::Nil,
		}
	}

	pub fn new_cons(car: LispObject, cdr: LispObject) -> Self {
		Self {
			kind: LispObjectKind::Cons(Cons::new(car, cdr)),
		}
	}

	pub fn new_empty_cons() -> Self {
		Self::new_cons(Self::new_nil(), Self::new_nil())
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cons {
	pub car: Box<LispObject>,
	pub cdr: Box<LispObject>,
}

impl Cons {
	pub fn new(car: LispObject, cdr: LispObject) -> Self {
		Cons {
			car: Box::new(car),
			cdr: Box::new(cdr),
		}
	}

	pub fn empty() -> Self {
		Self::new(LispObject::new_nil(), LispObject::new_nil())
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum Func {
	/// Args -> Return
	Primitive(fn(LispObject) -> LispObject),
	/// Body, Environment, Args -> Return
	Closure(fn(LispObject, Environment, LispObject) -> LispObject),
}
