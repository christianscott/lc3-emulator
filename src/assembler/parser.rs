use crate::assembler::lexer::{Token, TokenKind};

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
    token_in_line: usize,
    line: usize,
}

#[allow(dead_code)]
impl Reader {
    fn from(tokens: Vec<Token>) -> Self {
        Reader {
            tokens,
            offset: 0,
            token_in_line: 0,
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

        if c.clone().map_or(false, |c| c.kind == TokenKind::Newline) {
            self.line += 1;
            self.token_in_line = 0;
        } else {
            self.token_in_line += 1;
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
            match token.kind {
                TokenKind::Directive(directive) => {
                    parser.parse_directive(&directive)?;
                    continue;
                }
                TokenKind::Symbol(_string) => continue,
                TokenKind::Number(_num) => continue,
                TokenKind::Comma => continue,
                TokenKind::Str(_string) => continue,
                TokenKind::Newline => continue,
            }
        }

        Ok(vec![])
    }

    fn parse_directive(&mut self, directive: &str) -> Result<(), ParseError> {
        match directive.to_lowercase().as_ref() {
            "fill" => {
                let literal = self.expect_literal()?;
            }
            _ => {
                return Err(ParseError {
                    message: format!("unrecognized directive: {}", directive),
                })
            }
        }

        Ok(())
    }

    fn expect_literal(&mut self) -> Result<Token, ParseError> {
        match self.reader.next() {
            Some(token) => match token.kind {
                TokenKind::Number(_) | TokenKind::Str(_) => Ok(token),
                _ => Err(ParseError {
                    message: String::from("expected a literal"),
                }),
            },
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
    fn fill_with_literal() {
        assert_eq!(
            parse(vec![Token::directive("fill", 0), Token::number(10, 0)]), //
            Ok(vec![])                                                      //
        );
        assert_eq!(
            parse(vec![Token::directive("fill", 0), Token::str("hey", 0)]), //
            Ok(vec![])                                                      //
        );
    }

    #[test]
    fn fill_without_literal() {
        assert_eq!(
            parse(vec![Token::directive("fill", 0), Token::number(10, 0)]),
            Ok(vec![])
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
}
