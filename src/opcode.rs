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
    Str(String),

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
    pub locals_size: usize,
}

#[derive(PartialEq, Clone, Debug)]
pub struct CompiledClass {
    pub fields: Vec<String>,
    pub methods: Vec<CompiledMethod>,
}

fn compile_expr(class_table: &ClassTable, locals: &Vec<String>, expr: &Expression) -> Vec<OpCode> {
    match expr {
        Expression::Get(name) if name == "this" => vec![This],
        Expression::Get(name) if locals.contains(name) => vec![GetV(locals.iter().position(|l| l == name).unwrap())],
        Expression::Get(name) if class_table.map.contains_key(name) => vec![New(class_table.map.get(name).unwrap().start)],
        Expression::Get(name) => panic!("No such class or variable: {name}"),
        Expression::GetF(obj, name) => {
            let mut ops = compile_expr(class_table, locals, &*obj);
            ops.push(GetF(name.to_owned()));
            ops
        },
        Expression::Call(obj, name, args) => {
            let mut ops = compile_expr(class_table, locals, &*obj);
            ops.extend(args.iter().flat_map(|a| compile_expr(class_table, locals, a)));
            ops.push(Call(name.to_owned(), args.len()));
            ops
        },
        Expression::Is(obj, class) => {
           if let Some(range) = class_table.map.get(class) {
                let mut ops = compile_expr(class_table, locals, &*obj);
                ops.push(Is(range.to_owned()));
                ops
           } else {
                panic!("No such class: {class}");
           }
        },
        Expression::Equals(a, b) => {
            let mut ops = compile_expr(class_table, locals, &*a);
            ops.extend(compile_expr(class_table, locals, &*b));
            ops.push(Equals);
            ops
        },
        Expression::String(value) => {
            vec![Str(value.to_owned())]
        },
    }
}

fn compile_block(class_table: &ClassTable, result: &mut Vec<OpCode>, locals: &mut Vec<String>, block: &Vec<Statement>) {
    for stmt in block {
        match stmt {
            Statement::SetV(name, value) => {
                let id = if let Some(id) = locals.iter().position(|l| l == name) {
                    id
                } else {
                    locals.push(name.to_owned());
                    locals.len() - 1
                };
                result.extend(compile_expr(class_table, &locals, value));
                result.push(SetV(id));
            },
            Statement::SetF(obj, name, value) => {
                result.extend(compile_expr(class_table, &locals, obj));
                result.extend(compile_expr(class_table, &locals, value));
                result.push(SetF(name.to_owned()));
            },
            Statement::Call(obj, name, args) => {
                result.extend(compile_expr(class_table, &locals, &*obj));
                result.extend(args.iter().flat_map(|a| compile_expr(class_table, &locals, a)));
                result.push(Call(name.to_owned(), args.len()));
                result.push(Pop);
            },
            Statement::Return(value) => {
                result.extend(compile_expr(class_table, &locals, value));
                result.push(Return);
            },
            Statement::If(condition, block) => {
                result.extend(compile_expr(class_table, &locals, condition));
                let jump_index = result.len();
                result.push(Pop);
                compile_block(class_table, result, locals, block);
                result[jump_index] = Jump(false, result.len())
            },
            Statement::While(condition, block) => {
                result.extend(compile_expr(class_table, &locals, condition));
                let jump_index = result.len();
                result.push(Pop);
                compile_block(class_table, result, locals, block);
                result.extend(compile_expr(class_table, &locals, condition));
                result.push(Jump(true, jump_index + 1));
                result[jump_index] = Jump(false, result.len())
            },
        }
    }
}

fn compile_method(class_table: &ClassTable, method: &Method) -> CompiledMethod {
    if let Some(body) = &method.body {
        let mut locals = method.params.to_owned();
        let mut compiled_body = Vec::new();
        compile_block(class_table, &mut compiled_body, &mut locals, &body);
        CompiledMethod {
            name: method.name.to_owned(),
            body: Some(compiled_body),
            locals_size: locals.len(),
        }
    } else {
        CompiledMethod {
            name: method.name.to_owned(),
            body: None,
            locals_size: 0,
        }
    }
}

pub fn compile(class_table: &ClassTable) -> Vec<CompiledClass> {
    let mut result: Vec<CompiledClass> = Vec::with_capacity(class_table.classes.len());
    
    for c in &class_table.classes {
        let (inherited_fields, inherited_methods) = if let Some(pname) = &c.parent {
            let parent = &result[class_table.map.get(pname).unwrap().start];
            (parent.fields.to_owned(), parent.methods.to_owned())
        } else {
            (vec![], vec![])
        };
        result.push(CompiledClass {
            fields: inherited_fields.into_iter().filter(|f| !c.own_fields.contains(f)).chain(c.own_fields.iter().map(String::to_owned)).collect(),
            methods: inherited_methods.into_iter().filter(|m| !c.own_methods.iter().any(|mm| mm.name == m.name)).chain(c.own_methods.iter().map(|m| compile_method(class_table, m))).collect(),
        });
    }

    result
}
