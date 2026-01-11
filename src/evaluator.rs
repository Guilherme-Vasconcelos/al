//! Module responsible for evaluating lisp objects.
//!
//! A few concepts for object evaluation:
//! - "Environment lookup" always follows a hierarchy. First the current environment is looked up, and if nothing is found then its parent is looked up, and so on.
//!
//! The rules for evaluation, given each kind of lisp object, are:
//! - Literal (numbers, nil, functions) -> evaluates to itself.
//! - Symbol -> evaluates to an environment lookup.
//! - Cons cells -> evaluates to a function call. The function that will be called will be decided by an environment
//!   lookup.

use std::error::Error;
use std::fmt;

use crate::environment::Env;
use crate::object::{Cons, Func, FuncallError, LispObjConsBuilder, LispObject, LispObjectKind};

#[derive(Debug, PartialEq, Eq)]
pub enum EvalError {
	NotAFunction,
	ImproperList,
	UnboundSymbol,
	Runtime(FuncallError),
}

impl Error for EvalError {}

impl fmt::Display for EvalError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		match self {
			Self::NotAFunction => write!(f, "not a function"),
			Self::ImproperList => write!(f, "list is improper (does not end with NIL)"),
			Self::UnboundSymbol => write!(f, "unbound symbol"),
			Self::Runtime(fnclerr) => write!(f, "{fnclerr}"),
		}
	}
}

fn flat_eval_each(env: Env, list: &LispObject) -> Result<LispObject, EvalError> {
	assert!(matches!(list.kind, LispObjectKind::Cons(_)));

	let mut builder = LispObjConsBuilder::new();
	let mut iterator = list.into_iter();
	for obj in &mut iterator {
		let evaluated = eval(env.clone(), obj)?;
		builder.push(evaluated);
	}

	if iterator.is_proper() {
		Ok(builder.build())
	} else {
		Err(EvalError::ImproperList)
	}
}

pub fn eval(env: Env, obj: &LispObject) -> Result<LispObject, EvalError> {
	match &obj.kind {
		LispObjectKind::Num(_) | LispObjectKind::Nil | LispObjectKind::Func(_) => Ok(obj.clone()),
		LispObjectKind::Sym(s) => {
			let envb = env.borrow();
			envb.hierarchical_get(s).ok_or(EvalError::UnboundSymbol)
		}
		LispObjectKind::Cons(c) => eval_cons(env, c),
	}
}

fn eval_cons(env: Env, cons: &Cons) -> Result<LispObject, EvalError> {
	let car = &cons.car;
	let func = match &car.kind {
		LispObjectKind::Sym(s) => {
			let env = env.borrow();
			let funcobj = env.hierarchical_get(s).ok_or(EvalError::UnboundSymbol)?;

			match &funcobj.kind {
				LispObjectKind::Func(f) => funcobj,
				_ => return Err(EvalError::NotAFunction),
			}
		}

		_ => return Err(EvalError::NotAFunction),
	};

	let args = flat_eval_each(env, &cons.cdr)?;
	func.call_func(args).map_err(EvalError::Runtime)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::environment::new_global_env;

	#[test]
	fn test_eval_sym() {
		let env = new_global_env();
		let value = LispObject::new_num(123);
		let key = "foobar";

		{
			let mut env_b = env.borrow_mut();
			env_b.set(key, value.clone());
		}

		let sym_obj = LispObject::new_sym(key.to_owned());
		let result = eval(env.clone(), &sym_obj).unwrap();
		assert_eq!(result, value);
	}

	#[test]
	fn test_eval_sum_function() {
		let env = new_global_env();
		let obj = LispObjConsBuilder::from(vec![
			LispObject::new_sym("+".into()),
			LispObject::new_num(123),
			LispObject::new_num(57),
		])
		.build();
		let sum = eval(env, &obj).unwrap();
		assert_eq!(sum, LispObject::new_num(123 + 57));
	}

	#[test]
	fn test_malformed_sum_function() {
		let env = new_global_env();
		let obj = LispObjConsBuilder::from(vec![
			LispObject::new_sym("+".into()),
			LispObject::new_nil(),
			LispObject::new_num(57),
		])
		.build();
		let sum = eval(env, &obj);
		assert!(sum.is_err());
		assert_eq!(
			sum.unwrap_err(),
			EvalError::Runtime(FuncallError::WrongType)
		);
	}

	#[test]
	fn test_eval_atom() {
		let env = new_global_env();
		let obj = LispObject::new_num(57);
		let ev = eval(env, &obj).unwrap();
		assert_eq!(ev, obj);
	}

	#[test]
	fn test_eval_nonfunction_as_function() {
		let env = new_global_env();
		let key = "ashdoiawoipydad";
		{
			let mut envb = env.borrow_mut();
			envb.set(key, LispObject::new_nil());
		}
		let obj = LispObjConsBuilder::from(vec![
			LispObject::new_sym(key.into()),
			LispObject::new_num(57),
		])
		.build();
		let sum = eval(env, &obj);
		assert!(sum.is_err());
		assert_eq!(sum.unwrap_err(), EvalError::NotAFunction,);
	}
}
