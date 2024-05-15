use anyhow::{Result, Context, bail};

use self::TokenKind::*;

#[derive(PartialEq, Clone, Debug)]
pub enum TokenKind {
    Identifier(String),

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

pub fn tokenize(file_name: &str, input: &str) -> Result<Vec<Token>> {
    let mut result = Vec::new();
    let mut iter = input.chars().peekable();

    let mut line = 1;
    let mut column = 1;

    macro_rules! next {
        () => {
            if let Some(c) = iter.next() {
                if c == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
                Some(c)
            } else {
                None
            }
        }
    }

    macro_rules! require_next {
        () => {
            next!().with_context(|| "{file_name}:{line}:{column}: Unexpected end of file")?
        }
    }

    while let Some(c) = next!() {
        let maybe_kind = match c {
            ':' => Some(BlockStart),
            '.' => Some(Dot),
            ',' => Some(Comma),
            '=' => Some(EqualsSign),
            '(' => Some(OpeningParens),
            ')' => Some(ClosingParens),
            i if is_allowed_in_idents(i) => {
                let mut string = i.to_string();
                loop {
                    match iter.peek() {
                        Some(&s) if is_allowed_in_idents(s) => { 
                            next!();
                            string.push(s);
                        },
                        _ => break,
                    }
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
            '\'' => {
                let mut string = String::new();
                loop {
                    match require_next!() {
                        '\'' => break,
                        '\\' => string.push(match require_next!() {
                            'n' => '\n',
                            '\'' => '\'',
                            '\\' => '\\',
                            x => bail!("{file_name}:{line}:{column}: Invalid escape sequence: '\\{x}'"),
                        }),
                        s => string.push(s),
                    }
                }
                Some(Identifier(string))
            },
            '#' => {
                while iter.peek() != Some(&'\n') && iter.peek() != None {
                    next!();
                }
                None
            },
            w if w.is_whitespace() => None,
            c => bail!("{file_name}:{line}:{column}: Unexpected '{c}' character"),
        };

        if let Some(kind) = maybe_kind {
            result.push(Token { kind, line, column })
        }
    }

    Ok(result)
}
