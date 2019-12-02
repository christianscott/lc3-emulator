use crate::assembler::lexer::{Token, TokenKind};

use super::reader::Reader;
use std::collections::HashMap;
use std::iter::Extend;

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

struct Parser {
    reader: Reader<Token>,
    labels: HashMap<String, usize>,
}

impl Parser {
    fn parse(tokens: Vec<Token>) -> Result<Vec<Instruction>, ParseError> {
        let mut parser = Parser {
            reader: Reader::from(tokens, |t| t.kind == TokenKind::Newline),
            labels: HashMap::new(),
        };
        parser.find_labels();

        let mut instructions = vec![];
        while let Some(token) = parser.reader.next() {
            match token.kind {
                TokenKind::Directive(directive) => {
                    instructions.extend(parser.parse_directive(&directive)?);
                    continue;
                }
                TokenKind::Symbol(_string) => continue,
                TokenKind::Number(_num) => continue,
                TokenKind::Comma => continue,
                TokenKind::Str(_string) => continue,
                TokenKind::Newline => continue,
            }
        }

        Ok(instructions)
    }

    fn find_labels(&mut self) {
        while let Some(token) = self.reader.next() {
            if let TokenKind::Symbol(label) = token.kind {
                // if a symbol is at position 0 in the line, it's a label
                // rather than reference to a label
                if self.reader.item_in_line == 0 {
                    self.labels.insert(label, self.reader.line);
                }
            }
        }
        self.reader.reset();
    }

    fn parse_directive(&mut self, directive: &str) -> Result<Vec<Instruction>, ParseError> {
        match directive.to_lowercase().as_ref() {
            "fill" => Ok(vec![self.expect_number()?]),
            "stringz" => Ok(self.expect_string()?),
            _ => {
                return Err(ParseError {
                    message: format!("unrecognized directive: {}", directive),
                })
            }
        }
    }

    fn expect_number(&mut self) -> Result<u16, ParseError> {
        match self.reader.next() {
            Some(Token {
                kind: TokenKind::Number(num),
                ..
            }) => Ok(num),
            Some(_) => Err(ParseError {
                message: String::from("expected a number"),
            }),
            None => Err(ParseError {
                message: String::from("unexpected end of input"),
            }),
        }
    }

    fn expect_string(&mut self) -> Result<Vec<u16>, ParseError> {
        match self.reader.next() {
            Some(Token {
                kind: TokenKind::Str(string),
                ..
            }) => {
                let mut characters = Vec::new();

                characters.extend(string.chars().map(|c| c as u16));
                // null-terminate the string
                characters.push(0);

                Ok(characters)
            }
            Some(_) => Err(ParseError {
                message: String::from("expected a string literal"),
            }),
            None => Err(ParseError {
                message: String::from("unexpected end of input"),
            }),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<u16>, ParseError> {
    Parser::parse(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_directive() {
        assert_eq!(
            parse(vec![Token::directive(".bad", 0)]),
            Err(ParseError {
                message: String::from("unrecognized directive: .bad")
            }),
        );
    }

    #[test]
    fn fill_with_number() {
        assert_eq!(
            parse(vec![Token::directive("fill", 0), Token::number(10, 0)]),
            Ok(vec![10])
        );
    }

    #[test]
    fn fill_without_literal() {
        assert_eq!(
            parse(vec![Token::directive("fill", 0), Token::comma(0)]),
            Err(ParseError {
                message: String::from("expected a number")
            })
        )
    }

    #[test]
    fn fill_without_next_token() {
        assert_eq!(
            parse(vec![Token::directive("fill", 0)]),
            Err(ParseError {
                message: String::from("unexpected end of input"),
            })
        )
    }

    #[test]
    fn stringz_with_string_literal() {
        assert_eq!(
            parse(vec![Token::directive("stringz", 0), Token::str("a", 0)]),
            Ok(vec![97, 0])
        );
        assert_eq!(
            parse(vec![
                Token::directive("stringz", 0),
                Token::str("hello, world!", 0)
            ]),
            Ok(vec![
                104, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33, 0
            ])
        );
    }

    #[test]
    fn stringz_without_string_literal() {
        assert_eq!(
            parse(vec![Token::directive("stringz", 0), Token::number(10, 0)]),
            Err(ParseError {
                message: String::from("expected a string literal")
            })
        )
    }
}
