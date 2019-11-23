use std::borrow::ToOwned;
use std::u16;

#[derive(Debug, PartialEq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub character: usize,
}

impl LexError {
    pub fn pretty(self, filename: &str, source: &str) -> String {
        let line = source.lines().nth(self.line).unwrap();
        let line_indicator = format!("{} | ", self.line);
        let marker_line = format!(
            "{:width$}^ {}",
            "",
            self.message,
            width = line_indicator.len() + self.character + 1
        );
        format!(
            "{}:{}:{}\n\nlex error: {}\n{}{}\n{}",
            filename,
            self.line,
            self.character,
            self.message,
            line_indicator,
            line.replace('\t', " "),
            marker_line
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Directive(String),
    Symbol(String),
    Number(u16),
    Comma,
    Str(String),
    Newline,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub offset: usize,
}

#[allow(dead_code)]
impl Token {
    pub fn new(kind: TokenKind, offset: usize) -> Token {
        Token { kind, offset }
    }

    pub fn directive(string: &str, offset: usize) -> Token {
        Token::new(TokenKind::Directive(string.to_string()), offset)
    }

    pub fn symbol(string: &str, offset: usize) -> Token {
        Token::new(TokenKind::Symbol(string.to_string()), offset)
    }

    pub fn number(number: u16, offset: usize) -> Token {
        Token::new(TokenKind::Number(number), offset)
    }

    pub fn str(string: &str, offset: usize) -> Token {
        Token::new(TokenKind::Str(string.to_string()), offset)
    }

    pub fn newline(offset: usize) -> Token {
        Token::new(TokenKind::Newline, offset)
    }

    pub fn comma(offset: usize) -> Token {
        Token::new(TokenKind::Comma, offset)
    }
}

#[derive(Debug)]
struct Reader {
    source: Vec<char>,
    offset: usize,
    char_in_line: usize,
    line: usize,
}

impl Reader {
    fn from(source: &str) -> Self {
        Reader {
            source: source.chars().collect(),
            offset: 0,
            char_in_line: 0,
            line: 0,
        }
    }

    fn get(&self, index: usize) -> Option<char> {
        self.source.get(index).map(ToOwned::to_owned)
    }

    fn peek(&self) -> Option<char> {
        self.get(self.offset)
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        self.offset += 1;

        if c.map_or(false, |c| c == '\n') {
            self.line += 1;
            self.char_in_line = 0;
        } else {
            self.char_in_line += 1;
        }

        c
    }

    fn skip_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool + Copy,
    {
        while self.peek().map_or(false, predicate) {
            self.next();
        }
    }

    fn take_while<F>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool + Copy,
    {
        let mut chars = Vec::new();
        while self.peek().map_or(false, predicate) {
            match self.next() {
                Some(c) => chars.push(c),
                None => break,
            }
        }
        chars.iter().collect()
    }
}

struct Lexer {
    reader: Reader,
}

impl Lexer {
    fn from(source: &str) -> Self {
        Self {
            reader: Reader::from(source),
        }
    }

    fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            match self.reader.peek() {
                None => break,
                Some(c) => {
                    if let Some(token) = self.lex_char(c)? {
                        tokens.push(token);
                    }
                }
            }
        }

        Ok(tokens)
    }

    fn lex_char(&mut self, c: char) -> Result<Option<Token>, LexError> {
        if c == '\n' {
            let offset = self.reader.offset;
            self.reader.next();
            let token = Token {
                kind: TokenKind::Newline,
                offset,
            };
            return Ok(Some(token));
        }

        if c.is_whitespace() {
            self.reader.skip_while(char::is_whitespace);
            return Ok(None);
        }

        if c == ';' {
            self.reader.skip_while(|c| c != '\n');
            return Ok(None);
        }

        if c == 'x' {
            let offset = self.reader.offset;
            self.reader.next();

            let hex = self.reader.take_while(char::is_alphanumeric);
            let num = u16::from_str_radix(&hex, 16)
                .map_err(|e| self.error(format!("invalid hex literal 'x{}': {}", hex, e)))?;

            let token = Token {
                kind: TokenKind::Number(num),
                offset,
            };

            return Ok(Some(token));
        }

        if c == '#' {
            let offset = self.reader.offset;
            self.reader.next();
            return Ok(Some(self.lex_decimal(offset)?));
        }

        if c.is_numeric() || c == '-' {
            let offset = self.reader.offset;
            return Ok(Some(self.lex_decimal(offset)?));
        }

        if c == ',' {
            let offset = self.reader.offset;
            self.reader.next();
            let token = Token {
                kind: TokenKind::Comma,
                offset,
            };
            return Ok(Some(token));
        }

        if c == '.' {
            let offset = self.reader.offset;
            self.reader.next();
            let directive = self.reader.take_while(char::is_alphanumeric);
            let token = Token {
                kind: TokenKind::Directive(directive),
                offset,
            };
            return Ok(Some(token));
        }

        if c == '"' {
            let offset = self.reader.offset;
            self.reader.next();
            let string = self.reader.take_while(|c| c != '"');
            self.reader.next();
            let token = Token {
                kind: TokenKind::Str(string),
                offset,
            };
            return Ok(Some(token));
        }

        if c.is_alphabetic() {
            let offset = self.reader.offset;
            let symbol = self.reader.take_while(|c| c.is_alphanumeric() || c == '_');
            let token = Token {
                kind: TokenKind::Symbol(symbol),
                offset,
            };
            return Ok(Some(token));
        }

        Err(self.error(format!("unexpected char {}", c)))
    }

    fn lex_decimal(&mut self, offset: usize) -> Result<Token, LexError> {
        let negative = if self.reader.peek().map_or(false, |c| c == '-') {
            self.reader.next(); // skip the sign
            true
        } else {
            false
        };

        let dec = self.reader.take_while(char::is_alphanumeric);
        let num = u16::from_str_radix(&dec, 10)
            .map(|num| {
                if negative {
                    flip_sign_twos_complement(num)
                } else {
                    num
                }
            })
            .map_err(|e| self.error(format!("invalid decimal literal '{}': {}", dec, e)))?;
        let token = Token {
            kind: TokenKind::Number(num),
            offset,
        };
        Ok(token)
    }

    fn error(&self, message: String) -> LexError {
        LexError {
            message,
            line: self.reader.line,
            character: self.reader.char_in_line - 1,
        }
    }
}

/// flip the sign of an unsigned integer
fn flip_sign_twos_complement(n: u16) -> u16 {
    !(n - 1)
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    Lexer::from(source).lex()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignores_whitespace() {
        assert_eq!(lex(" \n\r\t"), Ok(vec![]));
    }

    #[test]
    fn test_ignores_comments() {
        assert_eq!(lex("; this is a comment"), Ok(vec![]));
        assert_eq!(
            lex(".directive ; this is a comment"),
            Ok(vec![Token::directive("directive", 0)])
        );
        assert_eq!(
            lex(".label\n ; this is a comment"),
            Ok(vec![Token::directive("label", 0), Token::newline(6)])
        );
    }

    #[test]
    fn test_continues_after_comments() {
        assert_eq!(
            lex("; a\n.directive"),
            Ok(vec![Token::newline(3), Token::directive("directive", 4)])
        );
    }

    #[test]
    fn test_lex_directive() {
        assert_eq!(
            lex(".directive"),
            Ok(vec![Token::directive("directive", 0)])
        );
        assert_eq!(
            lex("    .directive"),
            Ok(vec![Token::directive("directive", 4)])
        );
        assert_eq!(
            lex("\n.directive"),
            Ok(vec![Token::newline(0), Token::directive("directive", 1)])
        );
        assert_eq!(
            lex(".d1\n.d2"),
            Ok(vec![
                Token::directive("d1", 0),
                Token::newline(3),
                Token::directive("d2", 4)
            ])
        );
    }

    #[test]
    fn test_lex_symbol() {
        assert_eq!(lex("sym"), Ok(vec![Token::symbol("sym", 0)]));
        assert_eq!(
            lex("s1\ns2"),
            Ok(vec![
                Token::symbol("s1", 0),
                Token::newline(2),
                Token::symbol("s2", 3)
            ])
        );
    }

    #[test]
    fn test_lex_hex() {
        assert_eq!(lex("x0"), Ok(vec![Token::number(0, 0)]));
        assert_eq!(lex("xFFFF"), Ok(vec![Token::number(0xFFFF, 0)]));
        assert_eq!(
            lex("xG"),
            Err(LexError {
                message: "invalid hex literal 'xG': invalid digit found in string".to_string(),
                line: 0,
                character: 1,
            })
        );
    }

    #[test]
    fn test_lex_decimal() {
        assert_eq!(lex("#0"), Ok(vec![Token::number(0, 0)]));
        assert_eq!(lex("#1000"), Ok(vec![Token::number(1000, 0)]));
        assert_eq!(
            lex("#-1"),
            Ok(vec![Token::number(0b1111_1111_1111_1111, 0)])
        );
        assert_eq!(
            lex("#G"),
            Err(LexError {
                message: "invalid decimal literal 'G': invalid digit found in string".to_string(),
                line: 0,
                character: 1,
            })
        );
    }

    #[test]
    fn test_lex_strings() {
        assert_eq!(lex("\"hello\""), Ok(vec![Token::str("hello", 0)]));
    }

    #[test]
    fn test_real_asm() {
        assert_eq!(
            lex(".orig x3000"),
            Ok(vec![Token::directive("orig", 0), Token::number(0x3000, 6)])
        );
        assert_eq!(
            lex("	.FILL BAD_INT	; x01"),
            Ok(vec![
                Token::directive("FILL", 1),
                Token::symbol("BAD_INT", 7)
            ])
        );
        assert_eq!(
            lex("LD R0, MPR_INIT"),
            Ok(vec![
                Token::symbol("LD", 0),
                Token::symbol("R0", 3),
                Token::comma(5),
                Token::symbol("MPR_INIT", 7)
            ])
        );
        assert_eq!(
            lex("mystring .STRINGZ \"hello\""),
            Ok(vec![
                Token::symbol("mystring", 0),
                Token::directive("STRINGZ", 9),
                Token::str("hello", 18)
            ])
        );
    }
}
