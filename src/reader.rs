//! A few special notes about the reader:
//!
//! '()', i.e. empty list. This is the only case where the reader may produce a `LispObject` NIL
//! rather than letting the evaluator resolve it later, because there's no other valid way to
//! represent it as it is not a valid cons cell.
//! In other words, the following inputs are treated differently:
//! - `nil` -> a symbol that is resolved to NIL in runtime.
//! - `()` -> NOT a valid cons cell that gets resolved to NIL by the reader.
//! - `(nil . nil)` -> a fully valid cons cell whose CAR is the symbol nil and CDR is the symbol nil.
//!
//! The reader does not ever produce a Func `LispObject`, only Cons or atoms. Whether a Cons will get resolved
//! to a Func later or not is up to the evaluator. In other words, Cons is a data that has not yet
//! been evaluated, and Func is a Cons that has been evaluated to a Function.

use std::error::Error;
use std::fmt;

use crate::cursor::Cursor;
use crate::object::{LispObject, LispObjectKind};
use crate::tokenizer::{Token, TokenKind};

pub fn parse_tokens(tokens: &[Token]) -> Result<Vec<LispObject>, ReadingError> {
	Reader::new(tokens).collect()
}

#[derive(PartialEq, Eq, Debug)]
pub enum ReadingError {
	LeadingParenClose,
	IncompleteList,
}

impl fmt::Display for ReadingError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::LeadingParenClose => write!(f, "an object cannot begin with ')'"),
			Self::IncompleteList => write!(f, "list was started with '(' but has not been closed"),
		}
	}
}

impl Error for ReadingError {}

struct Reader<'a> {
	cursor: Cursor<'a, Token>,
}

impl<'a> Reader<'a> {
	pub const fn new(input: &'a [Token]) -> Self {
		Self {
			cursor: Cursor::new(input),
		}
	}

	fn parse_obj(&mut self) -> Result<LispObject, ReadingError> {
		let tok = self
			.cursor
			.get()
			.expect("parse_obj should not be called if there is no obj to parse");

		match &tok.kind {
			TokenKind::Num(_n) => Ok(make_atom_from_token(tok)),
			TokenKind::Sym(_s) => Ok(make_atom_from_token(tok)),
			TokenKind::ParenOpen => {
				if self
					.cursor
					.peek()
					.is_some_and(|t| t.kind == TokenKind::ParenClose)
				{
					let obj = LispObject::new_nil();
					self.cursor.advance();
					Ok(obj)
				} else {
					let mut obj = LispObject::new_empty_cons();
					self.parse_obj_in_list(&mut obj)?;
					Ok(obj)
				}
			}
			TokenKind::ParenClose => Err(ReadingError::LeadingParenClose),
		}
	}

	/// Parse the next object, but knowing we are inside a list.
	/// Therefore, we know the object we are about to parse will become the parent's CAR,
	/// and we may need to create a new cons cell that will become the parent's CDR (unless
	/// we are parsing the last object of the list, in which case we leave it as NIL).
	fn parse_obj_in_list(&mut self, parent: &mut LispObject) -> Result<(), ReadingError> {
		assert!(matches!(parent.kind, LispObjectKind::Cons(_)));

		// CAR
		let tok = self.cursor.get().ok_or(ReadingError::IncompleteList)?;
		let obj = match &tok.kind {
			TokenKind::Num(_n) => make_atom_from_token(tok),
			TokenKind::Sym(_s) => make_atom_from_token(tok),
			TokenKind::ParenOpen => {
				let tok = self.cursor.peek().ok_or(ReadingError::IncompleteList)?;
				if tok.kind == TokenKind::ParenClose {
					self.cursor.advance();
					LispObject::new_nil()
				} else {
					let mut cons = LispObject::new_empty_cons();
					self.parse_obj_in_list(&mut cons)?;
					cons
				}
			}
			TokenKind::ParenClose => {
				panic!("parse_obj_in_list must not be called when the list has ended")
			}
		};
		if let LispObjectKind::Cons(c) = &mut parent.kind {
			*c.car = obj;
		} else {
			unreachable!();
		}

		// CDR
		let tok = self.cursor.peek().ok_or(ReadingError::IncompleteList)?;
		match tok.kind {
			TokenKind::ParenClose => self.cursor.advance(),
			_ => {
				if let LispObjectKind::Cons(c) = &mut parent.kind {
					assert!(
						c.cdr.kind == LispObjectKind::Nil,
						"parent has an initialized CDR"
					);

					*c.cdr = LispObject::new_empty_cons();
					self.parse_obj_in_list(&mut c.cdr)?;
				} else {
					unreachable!();
				}
			}
		}

		Ok(())
	}
}

impl Iterator for Reader<'_> {
	type Item = Result<LispObject, ReadingError>;

	fn next(&mut self) -> Option<Self::Item> {
		self.cursor.peek()?;
		Some(self.parse_obj())
	}
}

fn make_atom_from_token(tok: &Token) -> LispObject {
	match &tok.kind {
		TokenKind::Num(n) => LispObject::new_num(*n),
		TokenKind::Sym(s) => LispObject::new_sym(s.to_owned()),
		TokenKind::ParenOpen | TokenKind::ParenClose => {
			panic!("make_atom_from_token should only be called for atoms")
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::object::LispObjConsBuilder;
	use crate::tokenizer::parse_bytes;

	fn build_objs_from_src(src: &str) -> Result<Vec<LispObject>, ReadingError> {
		let tokens = parse_bytes(src.as_bytes()).unwrap();
		parse_tokens(&tokens)
	}

	#[test]
	fn test_standalone_atoms() {
		let input = "5\n6\nfoobar\n";
		let objs = build_objs_from_src(input).unwrap();

		assert_eq!(
			objs,
			vec![
				LispObject::new_num(5),
				LispObject::new_num(6),
				LispObject::new_sym("foobar".to_string()),
			]
		);
	}

	#[test]
	fn test_empty_list() {
		let input = "()";
		let objs = build_objs_from_src(input).unwrap();

		assert_eq!(objs, vec![LispObject::new_nil()]);
	}

	#[test]
	fn test_empty_list_within_list() {
		let input = "(())";
		let objs = build_objs_from_src(input).unwrap();

		assert_eq!(
			objs,
			vec![LispObject::new_cons(
				LispObject::new_nil(),
				LispObject::new_nil()
			)]
		);
	}

	#[test]
	fn test_mega_nested_list() {
		let input = "(((1)))";
		let objs = build_objs_from_src(input).unwrap();

		assert_eq!(
			objs,
			vec![LispObject::new_cons(
				LispObject::new_cons(
					LispObject::new_cons(LispObject::new_num(1), LispObject::new_nil(),),
					LispObject::new_nil(),
				),
				LispObject::new_nil(),
			)]
		);
	}

	#[test]
	fn test_empty_list_and_other_atoms_within_list() {
		let input = "(() 1 ())";
		let objs = build_objs_from_src(input).unwrap();

		assert_eq!(
			objs,
			vec![LispObject::new_cons(
				LispObject::new_nil(),
				LispObject::new_cons(
					LispObject::new_num(1),
					LispObject::new_cons(LispObject::new_nil(), LispObject::new_nil(),)
				)
			)]
		);
	}

	#[test]
	fn test_list_with_one_atom() {
		let input = "(1)";
		let objs = build_objs_from_src(input).unwrap();

		assert_eq!(
			objs,
			vec![LispObject::new_cons(
				LispObject::new_num(1),
				LispObject::new_nil(),
			)]
		);
	}

	#[test]
	fn test_list_with_multiple_atoms() {
		let input = "(+ 1 test-sym)";
		let objs = build_objs_from_src(input).unwrap();

		assert_eq!(
			objs,
			vec![LispObject::new_cons(
				LispObject::new_sym("+".to_owned()),
				LispObject::new_cons(
					LispObject::new_num(1),
					LispObject::new_cons(
						LispObject::new_sym("test-sym".to_owned()),
						LispObject::new_nil()
					)
				)
			)]
		);
	}

	#[test]
	fn test_list_with_atoms_and_sublists() {
		let input = "((1 2 3) (4 5 (6 7) (8)))";
		let objs = build_objs_from_src(input).unwrap();

		let expected = vec![
			LispObjConsBuilder::from(vec![
				LispObjConsBuilder::from(vec![
					LispObject::new_num(1),
					LispObject::new_num(2),
					LispObject::new_num(3),
				])
				.build(),
				LispObjConsBuilder::from(vec![
					LispObject::new_num(4),
					LispObject::new_num(5),
					LispObjConsBuilder::from(vec![LispObject::new_num(6), LispObject::new_num(7)])
						.build(),
					LispObjConsBuilder::from(vec![LispObject::new_num(8)]).build(),
				])
				.build(),
			])
			.build(),
		];

		assert_eq!(objs, expected,);
	}

	#[test]
	fn test_two_separate_toplevel_objs() {
		let input = "(1 2)\nhello\n";
		let objs = build_objs_from_src(input).unwrap();

		assert_eq!(
			objs,
			vec![
				LispObject::new_cons(
					LispObject::new_num(1),
					LispObject::new_cons(LispObject::new_num(2), LispObject::new_nil(),),
				),
				LispObject::new_sym("hello".to_owned()),
			]
		);
	}

	#[test]
	fn test_list_with_leading_paren_close() {
		let input = ")1 2 3)";
		let objs = build_objs_from_src(input);

		assert!(objs.is_err());
		assert!(objs.unwrap_err() == ReadingError::LeadingParenClose);
	}

	#[test]
	fn test_list_without_closing_paren() {
		let input = "(1 2";
		let objs = build_objs_from_src(input);

		assert!(objs.is_err());
		assert!(objs.unwrap_err() == ReadingError::IncompleteList);
	}
}
