mod lexer;
mod parser;
mod reader;

#[derive(Debug, Default, PartialEq)]
pub struct Executable {
    pub instructions: Vec<u16>,
}

pub fn assemble(filename: &str, source: &str) -> Result<Executable, String> {
    let tokens = lexer::lex(source).map_err(|err| err.pretty(filename, source))?;
    let instructions = parser::parse(tokens).map_err(|err| err.pretty())?;
    Ok(Executable { instructions })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_empty() {
        assert_eq!(
            assemble("empty.asm", ""),
            Ok(Executable {
                instructions: Vec::new()
            })
        );
    }
}
