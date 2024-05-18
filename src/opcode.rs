use anyhow::{Result, Context, bail};

use crate::syntax::*;
use crate::class_table::*;

use self::OpCode::*;

#[derive(PartialEq, Clone, Debug)]
pub enum OpCode {
    New(usize),
    GetV(usize),
    This,
    GetF(String),
    Call(String, usize), // Method name and arg count
    Is(TypeRange),
    Equals,

    SetV(usize),
    SetF(String),
    Return,
    Jump(bool, usize),

    Pop,
}

#[derive(PartialEq, Clone, Debug)]
pub struct CompiledMethod {
    pub name: String,
    pub body: Option<Vec<OpCode>>,
    pub params_count: usize,
    pub locals_size: usize,
}

#[derive(PartialEq, Clone, Debug)]
pub struct CompiledClass {
    pub fields: Vec<String>,
    pub methods: Vec<CompiledMethod>,
}

fn compile_expr(class_table: &ClassTable, result: &mut Vec<OpCode>, locals: &Vec<String>, expr: &Expression) -> Result<()> {
    match expr {
        Expression::Get(name) if name == "this" => result.push(This),
        Expression::Get(name) if locals.contains(name) => result.push(GetV(locals.iter().position(|l| l == name).unwrap())),
        Expression::Get(name) if class_table.map.contains_key(name) => result.push(New(class_table.map.get(name).unwrap().0)),
        Expression::Get(name) => bail!("Couldn't find a class or variable named '{name}'"),
        Expression::GetF(obj, name) => {
            compile_expr(class_table, result, locals, obj)?;
            result.push(GetF(name.to_owned()));
        },
        Expression::Call(obj, name, args) => {
            compile_expr(class_table, result, locals, obj)?;
            for a in args {
                compile_expr(class_table, result, locals, a)?;
            }
            result.push(Call(name.to_owned(), args.len()));
        },
        Expression::Is(obj, class) => {
           if let Some(range) = class_table.map.get(class) {
                compile_expr(class_table, result, locals, obj)?;
                result.push(Is(range.to_owned()));
           } else {
                bail!("Couldn't find a class named '{class}'");
           }
        },
        Expression::Equals(a, b) => {
            compile_expr(class_table, result, locals, a)?;
            compile_expr(class_table, result, locals, b)?;
            result.push(Equals);
        },
    }
    Ok(())
}

fn compile_block(class_table: &ClassTable, result: &mut Vec<OpCode>, locals: &mut Vec<String>, block: &Vec<Statement>) -> Result<()> {
    for stmt in block {
        match stmt {
            Statement::SetV(name, value) => {
                let id = if let Some(id) = locals.iter().position(|l| l == name) {
                    id
                } else {
                    locals.push(name.to_owned());
                    locals.len() - 1
                };
                compile_expr(class_table, result, locals, value)?;
                result.push(SetV(id));
            },
            Statement::SetF(obj, name, value) => {
                compile_expr(class_table, result, locals, obj)?;
                compile_expr(class_table, result, locals, value)?;
                result.push(SetF(name.to_owned()));
            },
            Statement::Call(obj, name, args) => {
                compile_expr(class_table, result, locals, obj)?;
                for a in args {
                    compile_expr(class_table, result, locals, a)?;
                }
                result.push(Call(name.to_owned(), args.len()));
                result.push(Pop);
            },
            Statement::Return(value) => {
                compile_expr(class_table, result, locals, value)?;
                result.push(Return);
            },
            Statement::If(condition, block) => {
                compile_expr(class_table, result, locals, condition)?;
                let jump_index = result.len();
                result.push(Pop);
                compile_block(class_table, result, locals, block)?;
                result[jump_index] = Jump(false, result.len())
            },
            Statement::While(condition, block) => {
                compile_expr(class_table, result, locals, condition)?;
                let jump_index = result.len();
                result.push(Pop);
                compile_block(class_table, result, locals, block)?;
                compile_expr(class_table, result, locals, condition)?;
                result.push(Jump(true, jump_index + 1));
                result[jump_index] = Jump(false, result.len())
            },
        }
    }
    Ok(())
}

fn compile_method(class_table: &ClassTable, method: &Method) -> Result<CompiledMethod> {
    if let Some(body) = &method.body {
        let mut locals = method.params.to_owned();
        let mut compiled_body = Vec::new();
        compile_block(class_table, &mut compiled_body, &mut locals, &body)?;
        Ok(CompiledMethod {
            name: method.name.to_owned(),
            body: Some(compiled_body),
            params_count: method.params.len(),
            locals_size: locals.len(),
        })
    } else {
        Ok(CompiledMethod {
            name: method.name.to_owned(),
            body: None,
            params_count: method.params.len(),
            locals_size: 0,
        })
    }
}

pub fn compile(class_table: &ClassTable) -> Result<Vec<CompiledClass>> {
    let mut result: Vec<CompiledClass> = Vec::with_capacity(class_table.classes.len());
    
    for c in &class_table.classes {
        let (inherited_fields, inherited_methods) = if let Some(pname) = &c.parent {
            let parent = &result[class_table.get_class_id(pname)?];
            (parent.fields.to_owned(), parent.methods.to_owned())
        } else {
            (vec![], vec![])
        };
        result.push(CompiledClass {
            fields: inherited_fields.into_iter().filter(|f| !c.own_fields.contains(f)).chain(c.own_fields.iter().map(String::to_owned)).collect(),
            methods: inherited_methods.into_iter().filter(|m| !c.own_methods.iter().any(|mm| mm.name == m.name)).chain(c.own_methods.iter().map(|m| compile_method(class_table, m).with_context(|| format!("Failed to compile method '{}.{}'", c.name, m.name))).collect::<Result<Vec<_>, _>>()?).collect(),
        });
    }

    Ok(result)
}
