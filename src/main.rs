use advrs::lexer::*;
use advrs::parser::*;

const CODE: &str = r#":
if op.equals("+"):
    return a.plus(b)
end
end""#;

fn main() {
    let parsed = parse_block(&mut tokenize(CODE).iter().peekable());
    println!("{parsed:?}")
}
