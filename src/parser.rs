use std::fmt::Display;

use crate::tokenizer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnbalancedBrackets,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unbalanced brackets")
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    ShiftRight,
    ShiftLeft,
    Increment,
    Decrement,
    Print,
    Read,
    JumpIfZero(u8),
    JumpIfNotZero(u8),
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Statement>, ParseError> {
    let mut depth = 0;
    let mut instructions: Vec<Statement> = vec![];

    // Have to use an explicit loop because my function wasn't being TCO'd for some reason
    for token in tokens.iter() {
        match *token {
            Token::MovePointerLeft => {
                instructions.push(Statement::ShiftLeft);
            }
            Token::MovePointerRight => {
                instructions.push(Statement::ShiftRight);
            }
            Token::Increment => {
                instructions.push(Statement::Increment);
            }
            Token::Decrement => {
                instructions.push(Statement::Decrement);
            }
            Token::Print => {
                instructions.push(Statement::Print);
            }
            Token::Read => {
                instructions.push(Statement::Read);
            }
            Token::LeftBracket => {
                instructions.push(Statement::JumpIfZero(depth));
                depth += 1;
            }

            Token::RightBracket => {
                depth -= 1;
                instructions.push(Statement::JumpIfNotZero(depth));
            }
        }
    }

    if depth != 0 {
        Err(ParseError::UnbalancedBrackets)
    } else {
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_jump_expression() {
        let got = parse(&vec![
            Token::Increment,
            Token::LeftBracket,
            Token::Decrement,
            Token::RightBracket,
        ]);
        assert_eq!(
            got,
            Ok(vec![
                Statement::Increment,
                Statement::JumpIfZero(0),
                Statement::Decrement,
                Statement::JumpIfNotZero(0)
            ])
        );
    }

    #[test]
    fn nested_jump_expressions() {
        let got = parse(&vec![
            Token::Increment,
            Token::LeftBracket,
            Token::LeftBracket,
            Token::MovePointerLeft,
            Token::RightBracket,
            Token::Decrement,
            Token::RightBracket,
        ]);

        assert_eq!(
            got,
            Ok(vec![
                Statement::Increment,
                Statement::JumpIfZero(0),
                Statement::JumpIfZero(1),
                Statement::ShiftLeft,
                Statement::JumpIfNotZero(1),
                Statement::Decrement,
                Statement::JumpIfNotZero(0)
            ])
        )
    }

    #[test]
    fn unbalanced_expression() {
        let got = parse(&vec![
            Token::Increment,
            Token::LeftBracket,
            Token::Decrement,
        ]);
        assert_eq!(got, Err(ParseError::UnbalancedBrackets));
    }
}
