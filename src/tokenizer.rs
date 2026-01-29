use std::error::Error;
use std::fmt;

use crate::cursor::Cursor;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
	ParenOpen,
	ParenClose,
	Num(i64),
	Sym(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
	pub kind: TokenKind,
}

pub fn parse_bytes(buf: &[u8]) -> Result<Vec<Token>, TokenizationError> {
	Tokenizer::new(buf).collect()
}

#[derive(PartialEq, Eq, Debug)]
pub enum TokenizationError {
	MalformedNumber,
	UnsupportedBigNumber,
	UnfinishedComment,
}

impl fmt::Display for TokenizationError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::MalformedNumber => write!(f, "malformed number"),
			Self::UnsupportedBigNumber => write!(f, "big numbers are not supported yet"),
			Self::UnfinishedComment => write!(f, "unfinished comment"),
		}
	}
}

impl Error for TokenizationError {}

impl From<std::num::ParseIntError> for TokenizationError {
	fn from(_: std::num::ParseIntError) -> Self {
		Self::UnsupportedBigNumber
	}
}

const fn is_sym_or_number_end_delimiter(c: u8) -> bool {
	// Having a parenthesis or a whitespace next to a symbol or number means that symbol or number has ended.
	c == b')' || c == b'(' || c.is_ascii_whitespace()
}

struct Tokenizer<'a> {
	cursor: Cursor<'a, u8>,
}

impl<'a> Tokenizer<'a> {
	const fn new(input: &'a [u8]) -> Self {
		Self {
			cursor: Cursor::new(input),
		}
	}

	fn parse_num(&mut self) -> Result<Token, TokenizationError> {
		assert!(self.cursor.peek().is_some_and(u8::is_ascii_digit));

		let mut buf = String::new();
		while let Some(c) = self.cursor.get() {
			if c.is_ascii_digit() {
				buf.push(*c as char);
			} else if is_sym_or_number_end_delimiter(*c) {
				self.cursor.rewind();
				break;
			} else {
				return Err(TokenizationError::MalformedNumber);
			}
		}

		let value = buf.parse()?;
		Ok(Token {
			kind: TokenKind::Num(value),
		})
	}

	fn parse_negative_num(&mut self) -> Result<Token, TokenizationError> {
		// FIXME: - is a valid name for a variable.

		assert!(self.cursor.peek().is_some_and(|c| *c == b'-'));
		self.cursor.advance();

		if let Some(c) = self.cursor.peek()
			&& *c == b'-'
		{
			return Err(TokenizationError::MalformedNumber);
		}

		let mut tok = self.parse_num()?;
		match &mut tok.kind {
			TokenKind::Num(x) => {
				*x = -(*x);
			}
			_ => panic!("expected parse_num's result to be a number"),
		}
		Ok(tok)
	}

	fn skip_comment(&mut self) -> Result<(), TokenizationError> {
		assert!(self.cursor.peek().is_some_and(|c| *c == b';'));
		self.cursor.advance();

		let mut maybe_c = self.cursor.peek();
		if maybe_c.is_none() || maybe_c.is_some_and(|c| *c != b';') {
			return Err(TokenizationError::UnfinishedComment);
		}

		while maybe_c.is_some_and(|c| *c != b'\n') {
			self.cursor.advance();
			maybe_c = self.cursor.peek();
		}
		if maybe_c.is_some() {
			self.cursor.advance(); // Skip the newline itself
		}

		Ok(())
	}

	fn parse_sym(&mut self) -> Token {
		assert!(
			self.cursor
				.peek()
				.is_some_and(|c| !is_sym_or_number_end_delimiter(*c))
		);

		let mut buf = String::new();
		while let Some(c) = self.cursor.get() {
			if is_sym_or_number_end_delimiter(*c) {
				self.cursor.rewind();
				break;
			}

			buf.push(*c as char);
		}
		Token {
			kind: TokenKind::Sym(buf),
		}
	}

	fn parse_par_op(&mut self) -> Token {
		assert!(self.cursor.get().is_some_and(|c| *c == b'('));
		Token {
			kind: TokenKind::ParenOpen,
		}
	}

	fn parse_par_cl(&mut self) -> Token {
		assert!(self.cursor.get().is_some_and(|c| *c == b')'));
		Token {
			kind: TokenKind::ParenClose,
		}
	}

	fn skip_whitespaces_or_comments(&mut self) -> Result<(), TokenizationError> {
		loop {
			let mut brk = true;

			while self.cursor.peek().is_some_and(u8::is_ascii_whitespace) {
				brk = false;
				self.cursor.advance();
			}
			while self.cursor.peek().is_some_and(|c| *c == b';') {
				brk = false;
				self.skip_comment()?;
			}

			if brk {
				break;
			}
		}

		Ok(())
	}
}

impl Iterator for Tokenizer<'_> {
	type Item = Result<Token, TokenizationError>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Err(e) = self.skip_whitespaces_or_comments() {
			return Some(Err(e));
		}

		let c = self.cursor.peek()?;
		Some(match c {
			b'-' => self.parse_negative_num(),
			c if c.is_ascii_digit() => self.parse_num(),
			b'(' => Ok(self.parse_par_op()),
			b')' => Ok(self.parse_par_cl()),
			_ => Ok(self.parse_sym()),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn build_tokens_from_src(src: &str) -> Result<Vec<Token>, TokenizationError> {
		let bytes: Vec<u8> = src.bytes().collect();
		parse_bytes(&bytes)
	}

	#[test]
	fn test_empty_list() {
		let input = "()";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::ParenClose
				}
			]
		);
	}

	#[test]
	fn test_flat_numeric_list() {
		let input = "(+ 1 2)";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::Sym("+".to_owned())
				},
				Token {
					kind: TokenKind::Num(1)
				},
				Token {
					kind: TokenKind::Num(2)
				},
				Token {
					kind: TokenKind::ParenClose
				},
			]
		);
	}

	#[test]
	fn test_nested_list() {
		let input = "(((42)))";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::Num(42)
				},
				Token {
					kind: TokenKind::ParenClose
				},
				Token {
					kind: TokenKind::ParenClose
				},
				Token {
					kind: TokenKind::ParenClose
				},
			]
		);
	}

	#[test]
	fn test_list_ending_with_symbol() {
		let input = "(1 2 w)";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::Num(1)
				},
				Token {
					kind: TokenKind::Num(2)
				},
				Token {
					kind: TokenKind::Sym("w".to_owned())
				},
				Token {
					kind: TokenKind::ParenClose
				}
			]
		);
	}

	#[test]
	fn test_list_with_malformed_number() {
		let input = "(1 2w2)";
		let tokens = build_tokens_from_src(input);

		assert!(tokens.is_err());
		assert!(tokens.unwrap_err() == TokenizationError::MalformedNumber);
	}

	#[test]
	fn test_list_with_huge_number() {
		// We may eventually reach a point where big nums are a supported feature, but for now
		// nums are limited to rust's i64.
		let input = "(1 213127398123698132698162912863)";
		let tokens = build_tokens_from_src(input);

		assert!(tokens.is_err());
		assert!(tokens.unwrap_err() == TokenizationError::UnsupportedBigNumber);
	}

	#[test]
	fn test_list_with_extra_whitespaces() {
		let input = "  ( 1   2 3   )     ";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::Num(1)
				},
				Token {
					kind: TokenKind::Num(2)
				},
				Token {
					kind: TokenKind::Num(3)
				},
				Token {
					kind: TokenKind::ParenClose
				},
			]
		);
	}

	#[test]
	fn test_empty_input() {
		let input = "";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(tokens, Vec::new());
	}

	#[test]
	fn test_symbol_with_special_chars() {
		let input = "hello->123";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![Token {
				kind: TokenKind::Sym("hello->123".to_owned())
			}]
		);
	}

	#[test]
	fn test_list_without_whitespaces() {
		let input = "(foo(bar))";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::Sym("foo".to_owned())
				},
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::Sym("bar".to_owned())
				},
				Token {
					kind: TokenKind::ParenClose
				},
				Token {
					kind: TokenKind::ParenClose
				}
			]
		);
	}

	#[test]
	fn test_zero() {
		let input = "0";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![Token {
				kind: TokenKind::Num(0)
			}]
		);
	}

	#[test]
	fn test_list_with_negative_numbers() {
		let input = "(-123 8 -90)";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::Num(-123)
				},
				Token {
					kind: TokenKind::Num(8)
				},
				Token {
					kind: TokenKind::Num(-90)
				},
				Token {
					kind: TokenKind::ParenClose
				},
			]
		);
	}

	#[test]
	fn test_variable_named_minus() {
		let input = "-";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![Token {
				kind: TokenKind::Sym("-".to_owned())
			}]
		);
	}

	#[test]
	fn test_double_negative_number() {
		let input = "--1";
		let tokens = build_tokens_from_src(input);

		assert!(tokens.is_err());
		assert!(tokens.unwrap_err() == TokenizationError::MalformedNumber);
	}

	#[test]
	fn test_consecutive_symbols() {
		let input = "(foo+bar)";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token {
					kind: TokenKind::ParenOpen
				},
				Token {
					kind: TokenKind::Sym("foo+bar".to_owned())
				},
				Token {
					kind: TokenKind::ParenClose
				},
			]
		);
	}

	#[test]
	fn test_comment_is_skipped() {
		let input = ";;This is a comment\n;; Another comment\nnil\n";
		let tokens = build_tokens_from_src(input).unwrap();

		assert_eq!(
			tokens,
			vec![Token {
				kind: TokenKind::Sym("nil".to_owned())
			},]
		);
	}

	#[test]
	fn test_malformed_comment_is_err() {
		let input = ";\n";
		let tokens = build_tokens_from_src(input);

		assert!(tokens.is_err());
		assert!(tokens.unwrap_err() == TokenizationError::UnfinishedComment);
	}

	#[test]
	fn test_mix_of_comments_and_spaces_is_skipped() {
		let input = "  \n;; Hello!\n  ;; Hii\n  +";
		let tokens = build_tokens_from_src(input).unwrap();
		assert_eq!(
			tokens,
			vec![Token {
				kind: TokenKind::Sym("+".to_owned())
			},]
		);
	}
}
