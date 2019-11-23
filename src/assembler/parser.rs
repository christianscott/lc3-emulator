use crate::assembler::lexer::Token::{self, *};

#[derive(Debug, PartialEq)]
pub struct ParseError {
	pub message: String,
}

impl ParseError {
	pub fn pretty(self) -> String {
		String::from("")
	}
}

#[derive(Debug)]
struct Reader {
	tokens: Vec<Token>,
	offset: usize,
	char_in_line: usize,
	line: usize,
}

impl Reader {
	fn from(tokens: Vec<Token>) -> Self {
		Reader {
			tokens,
			offset: 0,
			char_in_line: 0,
			line: 0,
		}
	}

	fn get(&self, index: usize) -> Option<Token> {
		self.tokens.get(index).map(ToOwned::to_owned)
	}

	fn peek(&self) -> Option<Token> {
		self.get(self.offset)
	}

	fn next(&mut self) -> Option<Token> {
		let c = self.peek();
		self.offset += 1;

		if c.clone().map_or(false, |c| c == Newline) {
			self.line += 1;
			self.char_in_line = 0;
		} else {
			self.char_in_line += 1;
		}

		c
	}

	fn skip_while<F>(&mut self, predicate: F)
	where
		F: Fn(Token) -> bool + Copy,
	{
		while self.peek().map_or(false, predicate) {
			self.next();
		}
	}

	fn take_while<F>(&mut self, predicate: F) -> Vec<Token>
	where
		F: Fn(Token) -> bool + Copy,
	{
		let mut chars = Vec::new();
		while self.peek().map_or(false, predicate) {
			match self.next() {
				Some(c) => chars.push(c),
				None => break,
			}
		}
		chars.iter().map(ToOwned::to_owned).collect()
	}
}

type Instruction = u16;

struct Parser {
	reader: Reader,
}

impl Parser {
	fn parse(tokens: Vec<Token>) -> Result<Vec<Instruction>, ParseError> {
		let mut parser = Parser {
			reader: Reader::from(tokens),
		};
		while let Some(token) = parser.reader.next() {
			match token {
				Directive(directive) => {
					parser.parse_directive(&directive)?;
					continue;
				}
				Symbol(_string) => continue,
				Number(_num) => continue,
				Comma => continue,
				Str(_string) => continue,
				Newline => continue,
			}
		}

		Ok(vec![])
	}

	fn parse_directive(&mut self, directive: &str) -> Result<(), ParseError> {
		match directive.to_lowercase().as_ref() {
			".fill" => {}
			_ => {
				return Err(ParseError {
					message: format!("unrecognized directive: {}", directive),
				})
			}
		}

		Ok(())
	}
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<u16>, ParseError> {
	Parser::parse(tokens)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn directive(d: &str) -> Token {
		Directive(String::from(d))
	}

	#[test]
	fn test_bad_directive() {
		assert_eq!(
			parse(vec![directive(".bad")]),
			Err(ParseError {
				message: String::from("unrecognized directive: .bad")
			}),
		);
	}
}
