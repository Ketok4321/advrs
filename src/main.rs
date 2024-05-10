use advrs::lexer::*;
use advrs::syntax::*;
use advrs::parser::*;
use advrs::class_tree::*;
use advrs::opcode::*;
use advrs::interpreter::*;

const CODE: &str = r#"
class Program extends Object:
    method main()
end

class Input extends Object:
    field program    
    
    method read()
end

class Output extends Object:
    field program    
    
    method write(text)
end

class Boolean extends Object:
    method not()
    
    method and(b)
    
    method or(b)
end

class True extends Boolean:
    method not():
        return False
    end
    
    method and(b):
        return b
    end
    
    method or(b):
        return True
    end
end

class False extends Boolean:
    method not():
        return True
    end
    
    method and(b):
        return False
    end
    
    method or(b):
        return b
    end
end

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

class Test extends Program:
    method main():
        return this.get(True.not())
    end

    method get(obj):
        if obj is Object:
            if obj is Program:
                return Program
            end
            if obj is Input:
                return Input
            end
            if obj is Output:
                return Output
            end
            if obj is Boolean:
                if obj is True:
                    return True
                end
                if obj is False:
                    return False
                end
                return Boolean
            end
        end
    end
end
"#;

fn main() {
    let mut classes = vec![
        Class {
            name: "Object".to_string(),
            is_abstract: false,
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
        Class {
            name: "String".to_string(),
            is_abstract: false,
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
        Class {
            name: "Null".to_string(),
            is_abstract: false,
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
    ];
    classes.extend(parse(tokenize(CODE)));
    let tree = ClassTree::create(&classes);
    println!("{:?}", tree.map);
    let compiled = compile(&tree);
    let res = run(&tree, &compiled, &compiled[3].methods[0], new(3), &vec![]);
    println!("{res:?}");
}
