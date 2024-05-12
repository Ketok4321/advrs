use std::{fs, env, io};

use advrs::lexer::*;
use advrs::syntax::*;
use advrs::parser::*;
use advrs::class_table::*;
use advrs::opcode::*;
use advrs::interpreter::*;
use advrs::gc::*;

fn main() {
    let mut classes = vec![
        Class {
            name: "Object".to_string(),
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
        Class {
            name: "String".to_string(),
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
        Class {
            name: "Null".to_string(),
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
    ];

    let code = fs::read_to_string(env::args().nth(1).expect("No file provided")).expect("No such file");

    classes.extend(parse(tokenize(&code)));
    let table = ClassTable::create(&classes);
    let compiled = compile(&table);
    let entrypoint = {
        let diff = table.program.end - table.program.start;
        match diff {
            1 => panic!("No class extending program"),
            2 => table.program.start + 1,
            _ => {
                println!("Choose which one to run:");
                for p in table.program.start+1..table.program.end {
                    println!("{}) {}", p - table.program.start, table.classes[p].name);
                }
                let mut inp = String::new();
                io::stdin().read_line(&mut inp).expect("Failed to read line");
                let n = inp.trim().parse::<usize>().expect("Expected an integer");
                assert!(n >= 1 && n < diff);
                table.program.start + n
            }
        }
    };
    let ctx = RunCtx {
        class_table: table,
        classes: compiled,
    };

    let mut stack = vec![Object::TRUE_NULL; 1024];
    let mut gc = GC::new(&stack[..] as *const [Object], 1024);
    stack[0] = Object::new(&ctx, &mut gc, entrypoint);

    let res = run(&ctx, &mut gc, &mut stack, ctx.classes[entrypoint].methods.iter().find(|m| m.name == "main").unwrap());

    println!("{}", res.class_name(&ctx.class_table));
}
