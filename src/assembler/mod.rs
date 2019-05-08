mod parser;

#[derive(Debug, Default, PartialEq)]
pub struct Executable {
    pub instructions: Vec<u16>,
}

pub(crate) fn assemble(source: &str) -> Result<Executable, String> {
    let _tokens = parser::parse(source)?;
    Ok(Default::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_empty() {
        assert_eq!(
            assemble(""),
            Ok(Executable {
                instructions: Vec::new()
            })
        );
    }
}