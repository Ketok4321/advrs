use advrs::lexer::*;
use advrs::parser::*;

const CODE: &str = r#"
b = Mirror.instantiate((this.input).read())
""#;

fn main() {
    let parsed = parse_expression(&mut tokenize(CODE).iter().peekable());
    println!("{parsed:?}")
}
