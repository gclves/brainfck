#[derive(Debug, PartialEq)]
pub enum Token {
    MovePointerLeft,
    MovePointerRight,
    Decrement,
    Increment,
    Print,
    Read,
    LeftBracket,
    RightBracket,
}

pub fn tokenize(s: &str) -> Vec<Token> {
    s.chars()
        .filter_map(|c| match c {
            '<' => Some(Token::MovePointerLeft),
            '>' => Some(Token::MovePointerRight),
            '+' => Some(Token::Increment),
            '-' => Some(Token::Decrement),
            '.' => Some(Token::Print),
            ',' => Some(Token::Read),
            '[' => Some(Token::LeftBracket),
            ']' => Some(Token::RightBracket),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_angled_bracket() {
        let got = tokenize(">");
        assert_eq!(got, vec![Token::MovePointerRight]);
    }

    #[test]
    fn left_angled_bracket() {
        let got = tokenize("<");
        assert_eq!(got, vec![Token::MovePointerLeft]);
    }

    #[test]
    fn increment_operator() {
        let got = tokenize("+");
        assert_eq!(got, vec![Token::Increment]);
    }

    #[test]
    fn print_instruction() {
        let got = tokenize(".");
        assert_eq!(got, vec![Token::Print]);
    }

    #[test]
    fn complex_expression() {
        let got = tokenize(">+++.<,[-THIS IS A COMMENT]");
        assert_eq!(
            got,
            vec![
                Token::MovePointerRight,
                Token::Increment,
                Token::Increment,
                Token::Increment,
                Token::Print,
                Token::MovePointerLeft,
                Token::Read,
                Token::LeftBracket,
                Token::Decrement,
                Token::RightBracket
            ]
        );
    }
}
