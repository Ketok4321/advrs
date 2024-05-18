use std::{io, env, path, fs};
use anyhow::{Result, Context, bail, ensure};

use advrs::lexer::*;
use advrs::syntax::*;
use advrs::parser::*;
use advrs::class_table::*;
use advrs::opcode::*;
use advrs::interpreter::*;
use advrs::gc::*;

fn main() -> Result<()> {
    let mut classes = vec![
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

    let path = if let Some(p) = env::args().nth(1) {
        p
    } else {
        bail!("Usage: {} [file]", env::args().nth(0).unwrap_or("adv".to_string()));
    };
    let path = path::Path::new(&path);

    let (metadata, pclasses) = parse_file(&path).with_context(|| "Failed to read program file")?;
    classes.extend(pclasses);

    for dep in metadata.dependencies {
        let (_, pclasses) = parse_file(&path.parent().unwrap().join(dep)).with_context(|| "Failed to read dependecy file")?;
        classes.extend(pclasses);
    }

    let table = ClassTable::create(&classes)?;
    let compiled = compile(&table)?;
    let entrypoint = {
        if let Some(entry_name) = metadata.entrypoint {
            table.get_class_id(&entry_name)
        } else {
            let diff = table.program.1 - table.program.0;
            match diff {
                0 => bail!("Program class not defined"),
                1 => bail!("No class extending program"),
                2 => Ok(table.program.0 + 1),
                _ => {
                    println!("Choose which program to run:");
                    for p in table.program.0+1..table.program.1 {
                        println!("{}) {}", p - table.program.0, table.classes[p].name);
                    }
                    let mut inp = String::new();
                    io::stdin().read_line(&mut inp)?;
                    let n = inp.trim().parse::<usize>()?;
                    ensure!(n >= 1 && n < diff, "Inputted number was not in range");
                    Ok(table.program.0 + n)
                }
            }
        }
    }.with_context(|| "Failed to find entrypoint")?;
    let mut stack = vec![Object::TRUE_NULL; 8192];
    let mut gc = GC::new(&stack[..] as *const [Object], 4096);
    let ctx = RunCtx::new(&mut gc, table, compiled, entrypoint);
    stack[0] = ctx.program_obj;

    run(&ctx, &mut gc, &mut IOManager::new(), &mut stack, ctx.classes[entrypoint].methods.iter().find(|m| m.name == "main").with_context(|| "The entrypoint class doesn't have a main method")?);

    Ok(())
}

fn parse_file(path: &path::Path) -> Result<(Metadata, Vec<Class>)> {
    Ok(parse(tokenize(path.to_str().with_context(|| "Failed to stringify path")?, &fs::read_to_string(&path)?)?))
}
