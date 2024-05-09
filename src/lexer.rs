use self::Token::*;

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Identifier(String),
    StringLiteral(String),

    BlockStart,
    BlockEnd,

    Is,
    Return,
    If,
    While,
    Class,
    Extends,
    Field,
    Method,

    Dot,
    Comma,
    EqualsSign,
    OpeningParens,
    ClosingParens,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut result = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(c) = iter.next() {
        match c {
            ':' => result.push(BlockStart),
            '.' => result.push(Dot),
            ',' => result.push(Comma),
            '=' => result.push(EqualsSign),
            '(' => result.push(OpeningParens),
            ')' => result.push(ClosingParens),
            '"' => {
                let mut string = String::new();
                loop {
                    match iter.next() {
                        Some('"') => break,
                        Some(s) => string.push(s),
                        None => panic!(),
                    }
                }
                result.push(StringLiteral(string))
            },
            i if i.is_alphanumeric() => {
                let mut string = i.to_string();
                loop {
                    match iter.peek() {
                        Some(s) if s.is_alphanumeric() => { 
                            string.push(*s);
                        },
                        _ => break,
                    }
                    _ = iter.next();
                }
                result.push(match &*string {
                    "end" => BlockEnd,
                    "is" => Is,
                    "return" => Return,
                    "if" => If,
                    "while" => While,
                    "class" => Class,
                    "extends" => Extends,
                    "field" => Field,
                    "method" => Method,
                    _ => Identifier(string),
                })
            },
            '#' => 
                while iter.peek() != Some(&'\n') && iter.peek() != None {
                    _ = iter.next();
                }
            w if w.is_whitespace() => (),
            _ => panic!(),
        };
    }

    result
}
