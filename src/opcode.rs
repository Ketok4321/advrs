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
    GetFI(usize),
    Call(String, usize), // Method name and arg count
    Is(TypeRange),
    Equals,

    SetV(usize),
    SetF(String),
    SetFI(usize),
    Return,
    Jump(bool, usize),

    Pop,
}

impl OpCode {
    pub fn stack_diff(&self) -> isize {
        match &self {
            New(_) => 1,
            GetV(_) => 1,
            This => 1,
            GetF(_) | GetFI(_) => 0,
            Call(_, argc) => 1 - *argc as isize - 1,
            Is(_) => 0,
            Equals => -1,
            SetV(_) => -1,
            SetF(_) | SetFI(_) => -1,
            Return => -1,
            Jump(_, _) => -1,
            Pop => -1,
        }
    }
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

fn optimize_body(this_fields: &Vec<String>, compiled_body: &mut Vec<OpCode>) -> Result<()> {
    for i in 0..compiled_body.len() {
        match &compiled_body[i] {
            GetF(name) => {
                if compiled_body[i - 1] == This {
                    compiled_body[i] = GetFI(this_fields.iter().position(|f| f == name).with_context(|| "No such field")?)
                }
            },
            SetF(name) => {
                let mut stack_diff = 0;
                let mut j = i;
                while stack_diff != 1 {
                    j -= 1;
                    stack_diff += compiled_body[j].stack_diff();
                }

                if compiled_body[j - 1] == This {
                    compiled_body[i] = SetFI(this_fields.iter().position(|f| f == name).with_context(|| format!("No such field {name}"))?)
                }
            },
            _ => (),
        }
    }
    Ok(())
}

fn compile_method(class_table: &ClassTable, method: &Method, this_fields: &Vec<String>) -> Result<CompiledMethod> {
    if let Some(body) = &method.body {
        let mut locals = method.params.to_owned();
        let mut compiled_body = Vec::new();
        compile_block(class_table, &mut compiled_body, &mut locals, &body)?;
        optimize_body(this_fields, &mut compiled_body)?;
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

fn inherit<T: ToOwned>(parent: &Vec<T>, child: &Vec<T>, get_name: fn(&T) -> &str) -> Vec<T::Owned> {
    parent.iter().map(|p| if let Some(c) = child.iter().find(|c| get_name(c) == get_name(p)) { c } else { p }).chain(child.iter().filter(|c| !parent.iter().any(|p| get_name(p) == get_name(c)))).map(ToOwned::to_owned).collect()
}

pub fn compile(class_table: &ClassTable) -> Result<Vec<CompiledClass>> {
    let mut result: Vec<CompiledClass> = Vec::with_capacity(class_table.classes.len());
    
    for c in &class_table.classes {
        let parent = c.parent.to_owned().map(|p| &result[class_table.get_class_id(&p).unwrap()]);

        let fields = if let Some(p) = parent { inherit(&p.fields, &c.own_fields, |f| f) } else { c.own_fields.to_owned() };
        let my_methods = c.own_methods.iter().map(|m| compile_method(class_table, m, &fields).with_context(|| format!("Failed to compile method '{}.{}'", c.name, m.name))).collect::<Result<Vec<_>, _>>()?;
        let methods = if let Some(p) = parent { inherit(&p.methods, &my_methods, |m| &m.name) } else { my_methods };

        result.push(CompiledClass {
            fields,
            methods,
        });
    }

    Ok(result)
}
