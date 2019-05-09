use crate::{assembler::lexer::Token, instructions::Instruction};

enum ParseState {
    Initial,
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Instruction>, String> {
    let _state = ParseState::Initial;
    for _token in tokens {}

    Ok(Default::default())
}

#[cfg(test)]
mod tests {}
