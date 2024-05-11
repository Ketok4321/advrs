use advrs::lexer::*;
use advrs::syntax::*;
use advrs::parser::*;
use advrs::class_table::*;
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

class Test extends Program:
    method main():
        return this.get(True.not().or(True))
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
    let table = ClassTable::create(&classes);
    let compiled = compile(&table);
    let entrypoint = table.program.start + 1;
    let ctx = RunCtx {
        class_table: table,
        classes: compiled,
    };
    let res = run(&ctx, ctx.classes[entrypoint].methods.iter().find(|m| m.name == "main").unwrap(), new(entrypoint), &vec![]);

    println!("{}", res.class_name(&ctx.class_table));
}
