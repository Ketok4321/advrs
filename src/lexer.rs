use regex::Regex;

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Identifier(String),
    OpeningParenthesis,
    ClosingParenthesis,
    Dot,
    Comma,
    EqualsSign,
    Is,
    Return,
    If,
    While,
    BlockStart,
    BlockEnd,
    String(String)
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let comment_re = Regex::new(r"(?m)#.*\n").unwrap();
    let preprocessed = comment_re.replace_all(input, "\n");

    let mut result = Vec::new();

    let token_re = Regex::new(concat!(
        r"(?P<ident>[A-z0-9]+)|",
        r"(?P<oppar>\()|",
        r"(?P<clpar>\))|",
        r"(?P<dot>\.)|",
        r"(?P<comma>,)|",
        r"(?P<equals>=)|",
        r"(?P<blockstart>:)|",
        r#"(?P<string>"[^"]*")"#,
        )).unwrap();

    for cap in token_re.captures_iter(preprocessed.as_str()) {
        let token = if cap.name("ident").is_some() {
            match cap.name("ident").unwrap() {
                "is" => Token::Is,
                "return" => Token::Return,
                "if" => Token::If,
                "while" => Token::While,
                "end" => Token::BlockEnd,
                ident => Token::Identifier(ident.to_string())
            }
        } else if cap.name("oppar").is_some() {
            Token::OpeningParenthesis
        } else if cap.name("clpar").is_some() {
            Token::ClosingParenthesis
        } else if cap.name("dot").is_some() {
            Token::Dot
        } else if cap.name("comma").is_some() {
            Token::Comma
        } else if cap.name("equals").is_some() {
            Token::EqualsSign
        } else if cap.name("blockstart").is_some() {
            Token::BlockStart
        } else if cap.name("string").is_some() {
            Token::String(cap.name("string").unwrap().to_string())
        } else {
            panic!("Invalid token");
        };

        result.push(token)
    }

    result
}
