use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
enum TokenKind {
    ParenOpen,
    ParenClose,
    Num(i64),
    Sym(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
}

pub fn parse_bytes(buf: Vec<u8>) -> Result<Vec<Token>, TokenizationError> {
    Tokenizer::new(buf).collect()
}

#[derive(PartialEq, Eq, Debug)]
pub enum TokenizationError {
    MalformedNumber,
    UnsupportedBigNumber,
}

impl fmt::Display for TokenizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedNumber => write!(f, "malformed number"),
            Self::UnsupportedBigNumber => write!(f, "big numbers are not supported yet"),
        }
    }
}

impl Error for TokenizationError {}

impl From<std::num::ParseIntError> for TokenizationError {
    fn from(_: std::num::ParseIntError) -> Self {
        Self::UnsupportedBigNumber
    }
}

struct Tokenizer {
    input: Vec<u8>,
    current: usize,
}

const fn is_sym_or_number_end_delimiter(c: u8) -> bool {
    // Having a parenthesis or a whitespace next to a symbol or number means that symbol or number has ended.
    c == b')' || c == b'(' || c.is_ascii_whitespace()
}

impl Tokenizer {
    const fn new(input: Vec<u8>) -> Self {
        Self { input, current: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.current).copied()
    }

    fn getc(&mut self) -> Option<u8> {
        let result = self.peek();
        self.advance();
        result
    }

    const fn advance(&mut self) {
        self.current += 1;
    }

    const fn rewind(&mut self) {
        self.current -= 1;
    }

    fn parse_num(&mut self) -> Result<Token, TokenizationError> {
        assert!(self.peek().is_some_and(|c| c.is_ascii_digit()));

        let mut buf = String::new();
        while let Some(c) = self.getc() {
            if c.is_ascii_digit() {
                buf.push(c as char);
            } else if is_sym_or_number_end_delimiter(c) {
                self.rewind();
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
        assert!(self.peek().is_some_and(|c| c == b'-'));
        self.getc();

        if let Some(c) = self.peek()
            && c == b'-'
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
        return Ok(tok);
    }

    fn parse_sym(&mut self) -> Token {
        assert!(
            self.peek()
                .is_some_and(|c| !is_sym_or_number_end_delimiter(c))
        );

        let mut buf = String::new();
        while let Some(c) = self.getc() {
            if is_sym_or_number_end_delimiter(c) {
                self.rewind();
                break;
            }

            buf.push(c as char);
        }
        Token {
            kind: TokenKind::Sym(buf),
        }
    }

    fn parse_par_op(&mut self) -> Token {
        assert!(self.getc().is_some_and(|c| c == b'('));
        Token {
            kind: TokenKind::ParenOpen,
        }
    }

    fn parse_par_cl(&mut self) -> Token {
        assert!(self.getc().is_some_and(|c| c == b')'));
        Token {
            kind: TokenKind::ParenClose,
        }
    }
}

impl Iterator for Tokenizer {
    type Item = Result<Token, TokenizationError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
            // Skip all whitespaces
            self.advance();
        }

        let c = self.peek()?;
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
        parse_bytes(bytes)
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
}
