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
    orig: Option<u16>,
    instructions: Vec<Instruction>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            reader: Reader::from(tokens, |t| t.kind == TokenKind::Newline),
            labels: HashMap::new(),
            instructions: Vec::new(),
            orig: None,
        }
    }

    fn parse(&mut self) -> Result<Vec<Instruction>, ParseError> {
        self.find_labels();

        while let Some(token) = self.reader.next() {
            match token.kind {
                TokenKind::Directive(directive) => {
                    self.parse_directive(&directive)?;
                    continue;
                }
                TokenKind::Symbol(_string) => continue,
                TokenKind::Number(_num) => continue,
                TokenKind::Comma => continue,
                TokenKind::Str(_string) => continue,
                TokenKind::Newline => continue,
            }
        }

        Ok(self.instructions.clone())
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

    fn parse_directive(&mut self, directive: &str) -> Result<(), ParseError> {
        match directive.to_lowercase().as_ref() {
            "fill" => {
                let num = self.expect_number()?;
                self.instructions.push(num);
            }
            "stringz" => {
                let string = self.expect_string()?;

                let mut null_terminated_chars = Vec::new();
                null_terminated_chars.extend(string.chars().map(|c| c as u16));
                // null-terminate the string
                null_terminated_chars.push(0);

                self.instructions.extend(null_terminated_chars);
            }
            "blkw" => {
                let num_reserved_slots = self.expect_number()?;
                let reserved = vec![0; num_reserved_slots as usize];
                self.instructions.extend(reserved);
            }
            "orig" => {
                let orig = self.expect_number()?;
                self.orig = Some(orig);
            }
            "end" => {
                // stop parsing by moving to end of reader
                // TODO: fix this awful hack
                self.reader.offset = std::usize::MAX;
            }
            _ => {
                return Err(ParseError {
                    message: format!("unrecognized directive: {}", directive),
                })
            }
        }

        Ok(())
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

    fn expect_string(&mut self) -> Result<String, ParseError> {
        match self.reader.next() {
            Some(Token {
                kind: TokenKind::Str(string),
                ..
            }) => Ok(string),
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
    Parser::new(tokens).parse()
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

    #[test]
    fn stringz_without_next_token() {
        assert_eq!(
            parse(vec![Token::directive("stringz", 0)]),
            Err(ParseError {
                message: String::from("unexpected end of input")
            })
        )
    }

    #[test]
    fn orig() {
        let mut parser = Parser::new(vec![Token::directive("orig", 0), Token::number(0x3000, 0)]);
        assert_eq!(parser.parse(), Ok(vec![]));
        assert_eq!(parser.orig, Some(0x3000));
    }

    #[test]
    fn stop_parsing_after_end() {
        assert_eq!(
            parse(vec![
                Token::directive("fill", 0),
                Token::number(0, 0),
                Token::directive("end", 0),
                Token::directive("stringz", 0),
                Token::str("hey", 0),
            ]),
            Ok(vec![0])
        );
    }

    #[test]
    fn blkw() {
        assert_eq!(
            parse(vec![Token::directive("blkw", 0), Token::number(10, 0),]),
            Ok(vec![0; 10])
        );
    }
}
