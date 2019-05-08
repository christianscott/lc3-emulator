use std::borrow::ToOwned;
use std::u16;

#[derive(Debug, PartialEq)]
pub enum Token {
    Directive(String),
    Label(String),
    Number(u16),
    // Opcode(String),
    // Str(String),
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

    fn skip_while<F: Copy>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool,
    {
        while self.peek().map_or(false, predicate) {
            self.next();
        }
    }

    fn take_while<F: Copy>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool,
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

    fn lex(&mut self) -> Result<Vec<Token>, String> {
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

    fn lex_char(&mut self, c: char) -> Result<Option<Token>, String> {
        if c.is_whitespace() {
            self.reader.skip_while(char::is_whitespace);
            return Ok(None);
        }

        if c == ';' {
            self.reader.skip_while(|c| c != '\n');
            return Ok(None);
        }

        if c == 'x' {
            self.reader.next();
            let hex = self.reader.take_while(char::is_alphanumeric);
            let num = u16::from_str_radix(&hex, 16)
                .map_err(|e| format!("invalid hex literal 'x{}': {}", hex, e))?;
            return Ok(Some(Token::Number(num)));
        }

        if c == '#' {
            self.reader.next();
            let hex = self.reader.take_while(char::is_alphanumeric);
            let num = u16::from_str_radix(&hex, 10)
                .map_err(|e| format!("invalid decimal literal '#{}': {}", hex, e))?;
            return Ok(Some(Token::Number(num)));
        }

        if c == '.' {
            self.reader.next();
            let directive = self.reader.take_while(char::is_alphanumeric);
            return Ok(Some(Token::Directive(directive)));
        }

        if c.is_alphabetic() {
            let label = self.reader.take_while(|c| c.is_alphanumeric() || c == '_');
            return Ok(Some(Token::Label(label)));
        }

        // TODO: make an errors package
        Err(format!(
            "unexpected char {}\n{}:{}",
            c, self.reader.line, self.reader.char_in_line
        ))
    }
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    Lexer::from(source).lex()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directive(string: &str) -> Token {
        Token::Directive(string.to_string())
    }

    fn label(string: &str) -> Token {
        Token::Label(string.to_string())
    }

    fn number(number: u16) -> Token {
        Token::Number(number)
    }

    #[test]
    fn test_ignores_whitespace() {
        assert_eq!(lex(" \n\r\t"), Ok(vec![]));
    }

    #[test]
    fn test_ignores_comments() {
        assert_eq!(lex("; this is a comment"), Ok(vec![]));
        assert_eq!(
            lex(".directive ; this is a comment"),
            Ok(vec![directive("directive")])
        );
        assert_eq!(
            lex(".label\n ; this is a comment"),
            Ok(vec![directive("label")])
        );
    }

    #[test]
    fn test_continues_after_comments() {
        assert_eq!(
            lex("; this is a comment\n.directive"),
            Ok(vec![directive("directive")])
        );
    }

    #[test]
    fn test_lex_directive() {
        assert_eq!(lex(".directive"), Ok(vec![directive("directive")]));
        assert_eq!(lex("    .directive"), Ok(vec![directive("directive")]));
        assert_eq!(lex("\n.directive"), Ok(vec![directive("directive")]));
        assert_eq!(lex(".d1\n.d2"), Ok(vec![directive("d1"), directive("d2")]));
    }

    #[test]
    fn test_lex_label() {
        assert_eq!(lex("label"), Ok(vec![label("label")]));
        assert_eq!(lex("l1\nl2"), Ok(vec![label("l1"), label("l2")]));
    }

    #[test]
    fn test_lex_hex() {
        assert_eq!(lex("x0"), Ok(vec![number(0)]));
        assert_eq!(lex("xFFFF"), Ok(vec![number(0xFFFF)]));
        assert_eq!(
            lex("xG"),
            Err(String::from(
                "invalid hex literal 'xG': invalid digit found in string"
            ))
        );
    }

    #[test]
    fn test_lex_decimal() {
        assert_eq!(lex("#0"), Ok(vec![number(0)]));
        assert_eq!(lex("#1000"), Ok(vec![number(1000)]));
        assert_eq!(
            lex("#G"),
            Err(String::from(
                "invalid decimal literal '#G': invalid digit found in string"
            ))
        );
    }

    #[test]
    fn test_real_asm() {
        assert_eq!(
            lex(".orig x3000"),
            Ok(vec![directive("orig"), number(0x3000)])
        );
        assert_eq!(
            lex("	.FILL BAD_INT	; x01"),
            Ok(vec![directive("FILL"), label("BAD_INT")])
        );
    }
}
