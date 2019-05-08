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
}

impl<'a> Reader {
    fn from(source: &str) -> Self {
        Reader {
            source: source.chars().collect(),
            offset: 0,
        }
    }

    fn get(&self, index: usize) -> Option<char> {
        self.source.get(index).map(|c| c.to_owned())
    }

    fn peek(&self) -> Option<char> {
        self.get(self.offset)
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        self.offset += 1;
        c
    }

    fn skip_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool,
    {
        while self.peek().map_or(false, |c| predicate(c)) {
            self.next();
        }
    }

    fn take_while<F>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut chars = Vec::new();
        while self.peek().map_or(false, |c| predicate(c)) {
            match self.next() {
                Some(c) => chars.push(c),
                None => break,
            }
        }
        chars.iter().collect()
    }
}

pub fn parse(source: &str) -> Result<Vec<Token>, String> {
    let mut reader = Reader::from(source);
    let mut tokens = Vec::new();

    loop {
        match reader.peek() {
            None => break,
            Some(c) => {
                if c.is_whitespace() {
                    reader.skip_while(|c| c.is_whitespace());
                    continue;
                }

                if c == ';' {
                    reader.skip_while(|c| c != '\n');
                    continue;
                }

                if c == 'x' {
                    reader.next();
                    let hex = reader.take_while(|c| c.is_alphanumeric());
                    let num = u16::from_str_radix(&hex, 16)
                        .map_err(|e| format!("invalid hex literal 'x{}': {}", hex, e))?;
                    tokens.push(Token::Number(num));
                    continue;
                }

                if c == '#' {
                    reader.next();
                    let hex = reader.take_while(|c| c.is_alphanumeric());
                    let num = u16::from_str_radix(&hex, 10)
                        .map_err(|e| format!("invalid decimal literal '#{}': {}", hex, e))?;
                    tokens.push(Token::Number(num));
                    continue;
                }

                if c == '.' {
                    reader.next();
                    let directive = reader.take_while(|c| c.is_alphanumeric());
                    tokens.push(Token::Directive(directive));
                    continue;
                }

                if c.is_alphabetic() {
                    let label = reader.take_while(|c| c.is_alphanumeric() || c == '_');
                    tokens.push(Token::Label(label));
                    continue;
                }

                panic!("not implemented: {}", c);
            }
        }
    }

    Ok(tokens)
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
        assert_eq!(parse(" \n\r\t"), Ok(vec![]));
    }

    #[test]
    fn test_ignores_comments() {
        assert_eq!(parse("; this is a comment"), Ok(vec![]));
        assert_eq!(
            parse(".directive ; this is a comment"),
            Ok(vec![directive("directive")])
        );
        assert_eq!(
            parse(".label\n ; this is a comment"),
            Ok(vec![directive("label")])
        );
    }

    #[test]
    fn test_continues_after_comments() {
        assert_eq!(
            parse("; this is a comment\n.directive"),
            Ok(vec![directive("directive")])
        );
    }

    #[test]
    fn test_parse_directive() {
        assert_eq!(parse(".directive"), Ok(vec![directive("directive")]));
        assert_eq!(parse("    .directive"), Ok(vec![directive("directive")]));
        assert_eq!(parse("\n.directive"), Ok(vec![directive("directive")]));
        assert_eq!(
            parse(".d1\n.d2"),
            Ok(vec![directive("d1"), directive("d2")])
        );
    }

    #[test]
    fn test_parse_label() {
        assert_eq!(parse("label"), Ok(vec![label("label")]));
        assert_eq!(parse("l1\nl2"), Ok(vec![label("l1"), label("l2")]));
    }

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse("x0"), Ok(vec![number(0)]));
        assert_eq!(parse("xFFFF"), Ok(vec![number(0xFFFF)]));
        assert_eq!(
            parse("xG"),
            Err(String::from(
                "invalid hex literal 'xG': invalid digit found in string"
            ))
        );
    }

    #[test]
    fn test_parse_decimal() {
        assert_eq!(parse("#0"), Ok(vec![number(0)]));
        assert_eq!(parse("#1000"), Ok(vec![number(1000)]));
        assert_eq!(
            parse("#G"),
            Err(String::from(
                "invalid decimal literal '#G': invalid digit found in string"
            ))
        );
    }

    #[test]
    fn test_real_asm() {
        assert_eq!(
            parse(".orig x3000"),
            Ok(vec![directive("orig"), number(0x3000)])
        );
        assert_eq!(
            parse("	.FILL BAD_INT	; x01"),
            Ok(vec![directive("FILL"), label("BAD_INT")])
        );
    }
}
