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

type Instruction = u16;

struct Parser {}

impl Parser {
	fn parse(tokens: Vec<Token>) -> Result<Vec<Instruction>, ParseError> {
		for token in tokens {
			match token {
				Directive(directive) => {
					parse_directive(&directive)?;
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
}

fn parse_directive(directive: &str) -> Result<(), ParseError> {
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
