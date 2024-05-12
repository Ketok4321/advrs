use self::TokenKind::*;

#[derive(PartialEq, Clone, Debug)]
pub enum TokenKind {
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

#[derive(PartialEq, Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

fn is_allowed_in_idents(c: char) -> bool {
    c.is_alphanumeric() || match c {
        '_' | '+' | '-' | '*' | '/' => true,
        _ => false,
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut result = Vec::new();
    let mut iter = input.chars().peekable();

    let mut line = 1;
    let mut column = 1;

    while let Some(c) = iter.next() {
        let maybe_kind = match c {
            ':' => Some(BlockStart),
            '.' => Some(Dot),
            ',' => Some(Comma),
            '=' => Some(EqualsSign),
            '(' => Some(OpeningParens),
            ')' => Some(ClosingParens),
            '"' => {
                let mut string = String::new();
                loop {
                    match iter.next() {
                        Some('"') => break,
                        Some(s) => string.push(s),
                        None => panic!("Expected a string, found eof"),
                    }
                }
                Some(StringLiteral(string))
            },
            i if is_allowed_in_idents(i) => {
                let mut string = i.to_string();
                loop {
                    match iter.peek() {
                        Some(s) if is_allowed_in_idents(*s) => { 
                            string.push(*s);
                        },
                        _ => break,
                    }
                    _ = iter.next();
                }
                Some(match &*string {
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
            '#' => {
                while iter.peek() != Some(&'\n') && iter.peek() != None {
                    _ = iter.next();
                }
                None
            },
            '\n' => {
                line += 1;
                column = 0;
                None
            },
            w if w.is_whitespace() => None,
            c => panic!("Unexpected '{c}' character at {line}:{column}"),
        };

        if let Some(kind) = maybe_kind {
            result.push(Token { kind, line, column })
        }
        column += 1;
    }

    result
}
