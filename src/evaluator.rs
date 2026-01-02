use std::error::Error;
use std::fmt;

use crate::environment::Environment;
use crate::func::FuncallError;
use crate::object::{Func, LispObjConsBuilder, LispObject, LispObjectKind};

#[derive(Debug, PartialEq, Eq)]
pub enum EvalError {
	NotAFunction,
	ImproperList,
	Runtime(FuncallError),
}

impl Error for EvalError {}

impl fmt::Display for EvalError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		match self {
			Self::NotAFunction => write!(f, "not a function"),
			Self::ImproperList => write!(f, "list is improper (does not end with NIL)"),
			Self::Runtime(fnclerr) => write!(f, "{}", fnclerr),
		}
	}
}

fn flat_eval_each(env: &mut Environment, list: &LispObject) -> Result<LispObject, EvalError> {
	assert!(matches!(list.kind, LispObjectKind::Cons(_)));

	let mut builder = LispObjConsBuilder::new();
	let mut iterator = list.into_iter();
	for obj in &mut iterator {
		let evaluated = eval(env, obj)?;
		builder.push(evaluated);
	}

	if !iterator.is_proper() {
		Err(EvalError::ImproperList)
	} else {
		Ok(builder.build())
	}
}

pub fn eval(env: &mut Environment, obj: &LispObject) -> Result<LispObject, EvalError> {
	match &obj.kind {
		LispObjectKind::Num(_)
		| LispObjectKind::Sym(_)
		| LispObjectKind::Nil
		| LispObjectKind::Func(_) => Ok(obj.clone()),
		LispObjectKind::Cons(c) => {
			let car = &c.car;
			let func;
			match &car.kind {
				LispObjectKind::Sym(s) => {
					let func_obj = env.get(s).ok_or(EvalError::NotAFunction)?;
					if let LispObjectKind::Func(f) = &func_obj.kind {
						func = f.clone();
					} else {
						return Err(EvalError::NotAFunction);
					}
				}
				_ => return Err(EvalError::NotAFunction),
			}

			let args = flat_eval_each(env, &c.cdr)?;
			match func {
				Func::Primitive(p) => {
					let func_obj = LispObject::new_primitive(p);
					func_obj.call_func(args).map_err(EvalError::Runtime)
				}
				Func::Closure(_) => panic!("closure is not supported yet"),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::environment::Environment;

	#[test]
	fn test_eval_sum_function() {
		let mut env = Environment::new_global();
		let obj = LispObjConsBuilder::from(vec![
			LispObject::new_sym("+".into()),
			LispObject::new_num(123),
			LispObject::new_num(57),
		])
		.build();
		let sum = eval(&mut env, &obj).unwrap();
		assert_eq!(sum, LispObject::new_num(123 + 57));
	}

	#[test]
	fn test_malformed_sum_functio() {
		let mut env = Environment::new_global();
		let obj = LispObjConsBuilder::from(vec![
			LispObject::new_sym("+".into()),
			LispObject::new_sym("hi".into()),
			LispObject::new_num(57),
		])
		.build();
		let sum = eval(&mut env, &obj);
		assert!(sum.is_err());
		assert_eq!(
			sum.unwrap_err(),
			EvalError::Runtime(FuncallError::WrongType)
		);
	}

	#[test]
	fn test_eval_atom() {
		let mut env = Environment::new_global();
		let obj = LispObject::new_num(57);
		let ev = eval(&mut env, &obj).unwrap();
		assert_eq!(ev, obj);
	}

	#[test]
	fn test_eval_nonfunction_as_function() {
		let mut env = Environment::new_global();
		let obj = LispObjConsBuilder::from(vec![
			LispObject::new_sym("ashdoiawoipydad".into()),
			LispObject::new_num(57),
		])
		.build();
		let sum = eval(&mut env, &obj);
		assert!(sum.is_err());
		assert_eq!(sum.unwrap_err(), EvalError::NotAFunction,);
	}
}
