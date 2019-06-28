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

#[derive(Debug, PartialEq)]
pub enum Token {
    Directive(String),
    Symbol(String),
    Number(u16),
    Comma,
    Str(String),
    Newline,
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
            self.reader.next();
            return Ok(Some(Token::Newline));
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
            self.reader.next();
            let hex = self.reader.take_while(char::is_alphanumeric);
            let num = u16::from_str_radix(&hex, 16)
                .map_err(|e| self.error(format!("invalid hex literal 'x{}': {}", hex, e)))?;
            return Ok(Some(Token::Number(num)));
        }

        if c == '#' {
            self.reader.next();
            return Ok(Some(self.lex_decimal()?));
        }

        if c.is_numeric() || c == '-' {
            return Ok(Some(self.lex_decimal()?));
        }

        if c == ',' {
            self.reader.next();
            return Ok(Some(Token::Comma));
        }

        if c == '.' {
            self.reader.next();
            let directive = self.reader.take_while(char::is_alphanumeric);
            return Ok(Some(Token::Directive(directive)));
        }

        if c == '"' {
            self.reader.next();
            let string = self.reader.take_while(|c| c != '"');
            self.reader.next();
            return Ok(Some(Token::Str(string)));
        }

        if c.is_alphabetic() {
            let symbol = self.reader.take_while(|c| c.is_alphanumeric() || c == '_');
            return Ok(Some(Token::Symbol(symbol)));
        }

        Err(self.error(format!("unexpected char {}", c)))
    }

    fn lex_decimal(&mut self) -> Result<Token, LexError> {
        let negative = if self.reader.peek().map_or(false, |c| c == '-') {
            self.reader.next();
            true
        } else {
            false
        };

        let dec = self.reader.take_while(char::is_alphanumeric);
        let num = u16::from_str_radix(&dec, 10)
            .map(|num| {
                if negative {
                    !(num - 1) // two's complement representation
                } else {
                    num
                }
            })
            .map_err(|e| self.error(format!("invalid decimal literal '{}': {}", dec, e)))?;
        Ok(Token::Number(num))
    }

    fn error(&self, message: String) -> LexError {
        LexError {
            message,
            line: self.reader.line,
            character: self.reader.char_in_line - 1,
        }
    }
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    Lexer::from(source).lex()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directive(string: &str) -> Token {
        Token::Directive(string.to_string())
    }

    fn symbol(string: &str) -> Token {
        Token::Symbol(string.to_string())
    }

    fn number(number: u16) -> Token {
        Token::Number(number)
    }

    fn str(string: &str) -> Token {
        Token::Str(string.to_string())
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
            Ok(vec![directive("label"), Token::Newline])
        );
    }

    #[test]
    fn test_continues_after_comments() {
        assert_eq!(
            lex("; this is a comment\n.directive"),
            Ok(vec![Token::Newline, directive("directive")])
        );
    }

    #[test]
    fn test_lex_directive() {
        assert_eq!(lex(".directive"), Ok(vec![directive("directive")]));
        assert_eq!(lex("    .directive"), Ok(vec![directive("directive")]));
        assert_eq!(
            lex("\n.directive"),
            Ok(vec![Token::Newline, directive("directive")])
        );
        assert_eq!(
            lex(".d1\n.d2"),
            Ok(vec![directive("d1"), Token::Newline, directive("d2")])
        );
    }

    #[test]
    fn test_lex_symbol() {
        assert_eq!(lex("sym"), Ok(vec![symbol("sym")]));
        assert_eq!(
            lex("s1\ns2"),
            Ok(vec![symbol("s1"), Token::Newline, symbol("s2")])
        );
    }

    #[test]
    fn test_lex_hex() {
        assert_eq!(lex("x0"), Ok(vec![number(0)]));
        assert_eq!(lex("xFFFF"), Ok(vec![number(0xFFFF)]));
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
        assert_eq!(lex("#0"), Ok(vec![number(0)]));
        assert_eq!(lex("#1000"), Ok(vec![number(1000)]));
        assert_eq!(lex("#-1"), Ok(vec![number(0b1111_1111_1111_1111)]));
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
        assert_eq!(lex("\"hello\""), Ok(vec![str("hello")]));
    }

    #[test]
    fn test_real_asm() {
        assert_eq!(
            lex(".orig x3000"),
            Ok(vec![directive("orig"), number(0x3000)])
        );
        assert_eq!(
            lex("	.FILL BAD_INT	; x01"),
            Ok(vec![directive("FILL"), symbol("BAD_INT")])
        );
        assert_eq!(
            lex("LD R0, MPR_INIT"),
            Ok(vec![
                symbol("LD"),
                symbol("R0"),
                Token::Comma,
                symbol("MPR_INIT")
            ])
        );
        assert_eq!(
            lex("mystring .STRINGZ \"hello\""),
            Ok(vec![symbol("mystring"), directive("STRINGZ"), str("hello")])
        );
    }
}
