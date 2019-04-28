use std::u16;

#[derive(Debug, Default, PartialEq)]
pub struct Executable {
    pub instructions: Vec<u16>,
}

#[derive(Debug, PartialEq)]
enum Token {
    Directive(String),
    Number(u16),
    Label(String),
    Opcode(String),
    _Str(String),
}

fn lex(source: String) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars();
    let chars = chars.by_ref();

    loop {
        let c = chars.next();
        if c.is_none() {
            break;
        }
        let c = c.unwrap();

        if c.is_whitespace() {
            continue;
        }

        if c == ';' {
            // TODO: consume without wasteful collect
            let _comment: String = chars.take_while(|c| *c != '\n').collect();
            continue;
        }

        if c == '.' {
            let directive: String = chars.take_while(|c| c.is_alphanumeric()).collect();
            tokens.push(Token::Directive(directive));
            continue;
        }

        if c == 'x' {
            let hex: String = chars.take_while(|c| !c.is_whitespace()).collect();
            let number = u16::from_str_radix(&hex, 16)
                .map_err(|err| format!("bad hex literal: {}", err.to_string()))?;
            tokens.push(Token::Number(number));
            continue;
        }

        if c == '#' {
            let hex: String = chars.take_while(|c| !c.is_whitespace()).collect();
            let number = u16::from_str_radix(&hex, 10)
                .map_err(|err| format!("bad decimal literal: {}", err.to_string()))?;
            tokens.push(Token::Number(number));
            continue;
        }

        if c.is_alphabetic() {
            let word = format!("{}{}", c, chars.take_while(|c| c.is_alphanumeric()).collect::<String>());
            match word.to_uppercase().as_ref() {
                "ADD" | "AND" | "BR" | "JMP" | "JSR" | "LD" | "LDI" | "LDR" | "LEA" | "NOT"
                | "RTI" | "ST" | "STI" | "STR" | "TRAP" => {
                    tokens.push(Token::Opcode(word));
                }
                // otherwise it's a label
                _ => {
                    tokens.push(Token::Label(word));
                }
            }

            continue;
        }

        return Err(format!("unexpected token {}", c));
    }

    Ok(tokens)
}

pub fn assemble(source: String) -> Executable {
    let _tokens = lex(source);
    Default::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directive(string: &str) -> Token {
        Token::Directive(String::from(string))
    }

    fn number(number: u16) -> Token {
        Token::Number(number)
    }

    fn label(string: &str) -> Token {
        Token::Label(String::from(string))
    }

    #[test]
    fn test_lex_basic() {
        let input = String::from(".orig x3000");
        assert_eq!(lex(input), Ok(vec![directive("orig"), number(0x3000)]));
    }

    #[test]
    fn test_lex_with_comments() {
        let input = String::from(
            r#"
            ;;; comment
            .orig x3000 ; .comment #7
            ; comment
        "#,
        );
        assert_eq!(lex(input), Ok(vec![directive("orig"), number(0x3000)]));
    }

    #[test]
    fn test_lex_decimal() {
        let input = String::from("#3000");
        assert_eq!(lex(input), Ok(vec![number(3000)]));

        let input = String::from("#0");
        assert_eq!(lex(input), Ok(vec![number(0)]));

        let input = String::from("#0a");
        assert_eq!(
            lex(input),
            Err(String::from(
                "bad decimal literal: invalid digit found in string"
            ))
        );

        let input = String::from("#a0");
        assert_eq!(
            lex(input),
            Err(String::from(
                "bad decimal literal: invalid digit found in string"
            ))
        );
    }

    #[test]
    fn test_lex_hex() {
        let input = String::from("x3000");
        assert_eq!(lex(input), Ok(vec![number(0x3000)]));

        let input = String::from("x0");
        assert_eq!(lex(input), Ok(vec![number(0)]));

        let input = String::from("x0a");
        assert_eq!(lex(input), Ok(vec![number(0x0A)]));

        let input = String::from("xa0");
        assert_eq!(lex(input), Ok(vec![number(0xA0)]));

        let input = String::from("x0g");
        assert_eq!(
            lex(input),
            Err(String::from(
                "bad hex literal: invalid digit found in string"
            ))
        );

        let input = String::from("xg0");
        assert_eq!(
            lex(input),
            Err(String::from(
                "bad hex literal: invalid digit found in string"
            ))
        );
    }

    #[test]
    fn test_parse_label() {
        let input = String::from("op1 .fill b10000000");
        assert_eq!(
            lex(input),
            Ok(vec![label("op1"), directive("fill"), number(0b10000000)])
        )
    }

    #[test]
    fn test_assemble_empty() {
        assert_eq!(
            assemble(String::new()),
            Executable {
                instructions: Vec::new()
            }
        );
    }
}
