use super::lexer::Token;

enum ParseState {
    Initial,
}

pub fn parse(tokens: Vec<Token>) -> Result<(), String> {
    let _state = ParseState::Initial;
    for _token in tokens {}
    Ok(())
}

#[cfg(test)]
mod tests {}
