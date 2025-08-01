use std::{io, env, path, fs};
use anyhow::{Result, Context, bail, ensure};

use advrs::lexer::*;
use advrs::syntax::*;
use advrs::parser::*;
use advrs::class_table::*;
use advrs::opcode::*;
use advrs::interpreter::*;
use advrs::gc::*;
use advrs::stringifier::*;

fn main() -> Result<()> {
    let builtin_classes = vec![
        Class {
            name: "Object".to_string(),
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

    let path = if let Some(p) = env::args().nth(2) {
        p
    } else {
        bail!("Usage: {} [run|merge] [file]", env::args().nth(0).unwrap_or("adv".to_string()));
    };
    let path = path::Path::new(&path);

    let (metadata, mut classes) = parse_file(&path)?;

    for dep in &metadata.dependencies {
        let (_, dclasses) = parse_file(&path.parent().unwrap().join(dep))?;
        classes.extend(dclasses);
    }

    match env::args().nth(1).unwrap().as_str() {
        "run" => {
            let all_classes = [builtin_classes, classes].concat();

            let table = ClassTable::create(&all_classes)?;
            let compiled = compile(&table)?;
            let entrypoint = {
                match &metadata.entrypoints[..] {
                    [] => bail!("No entrypoint defined"),
                    [id] => Ok::<_, anyhow::Error>(table.get_class_id(&id)?),
                    list => {
                        println!("Choose entrypoint:");
                        for (i, ep) in list.iter().enumerate() {
                            println!("{}) {}", i + 1, ep);
                        }
                        let mut inp = String::new();
                        io::stdin().read_line(&mut inp)?;
                        let n = inp.trim().parse::<usize>()?;
                        ensure!(n >= 1 && n <= list.len(), "Inputted number was not in range");
                        Ok(table.get_class_id(&list[n - 1])?)
                    }
                }
            }.with_context(|| "Failed to find entrypoint")?;
            let mut stack = vec![Object::TRUE_NULL; 8192];
            let mut gc = GC::new(&stack[..] as *const [Object], 4096);
            let ctx = RunCtx::new(&mut gc, table, compiled, entrypoint);
            stack[0] = ctx.entrypoint;

            run(&ctx, &mut gc, &mut String::new(), &mut stack, ctx.classes[entrypoint].methods.iter().find(|m| m.name == "main").with_context(|| "The entrypoint class doesn't have a main method")?).with_context(|| "Runtime error")?;

        },
        "merge" => {
            let new_metadata = Metadata {
                dependencies: vec![],
                ..metadata
            };

            println!("{}", stringify(&new_metadata, &classes));
        },
        _ => bail!("You're using it wrong :<"),
    }
    Ok(())
}

fn parse_file(path: &path::Path) -> Result<(Metadata, Vec<Class>)> {
    let file_name = path.to_str().with_context(|| "Failed to stringify path")?;
    Ok(parse(file_name, tokenize(file_name, &fs::read_to_string(&path)?)?)?)
}
