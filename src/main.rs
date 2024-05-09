use advrs::lexer::*;
use advrs::parser::*;

const CODE: &str = r#"
class Cat extends Program:
    field input
    field output
    method main():
        this.input = Input
        this.output = Output

        (this.input).program = this
        (this.output).program = this

        input = (this.input).read()
        (this.output).write(input)
    end
end
"#;

fn main() {
    let parsed = parse(tokenize(CODE));
    println!("{parsed:?}")
}
