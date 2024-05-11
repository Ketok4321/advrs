use crate::syntax::*;
use crate::class_tree::*;

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
    pub is_abstract: bool,
    pub fields: Vec<String>,
    pub methods: Vec<CompiledMethod>,
}

fn compile_expr(class_tree: &ClassTree, locals: &Vec<String>, expr: &Expression) -> Vec<OpCode> {
    match expr {
        Expression::Get(name) if name == "this" => vec![This],
        Expression::Get(name) if locals.contains(name) => {
            if let Some(id) = locals.iter().position(|l| l == name) {
                vec![GetV(id)]
            } else {
                panic!();
            }
        },
        Expression::Get(name) => {
            if let Some(range) = class_tree.map.get(name) {
                vec![New(range.start)]
            } else {
                panic!();
            }
        },
        Expression::GetF(obj, name) => {
            let mut ops = compile_expr(class_tree, locals, &*obj);
            ops.push(GetF(name.to_owned()));
            ops
        },
        Expression::Call(obj, name, args) => {
            let mut ops = compile_expr(class_tree, locals, &*obj);
            ops.extend(args.iter().flat_map(|a| compile_expr(class_tree, locals, a)));
            ops.push(Call(name.to_owned(), args.len()));
            ops
        },
        Expression::Is(obj, class) => {
           if let Some(range) = class_tree.map.get(class) {
                let mut ops = compile_expr(class_tree, locals, &*obj);
                ops.push(Is(range.to_owned()));
                ops
           } else {
                panic!();
           }
        },
        Expression::Equals(a, b) => {
            let mut ops = compile_expr(class_tree, locals, &*a);
            ops.extend(compile_expr(class_tree, locals, &*b));
            ops.push(Equals);
            ops
        },
        Expression::String(value) => {
            vec![Str(value.to_owned())]
        },
    }
}

fn compile_block(class_tree: &ClassTree, result: &mut Vec<OpCode>, locals: &mut Vec<String>, block: &Vec<Statement>) -> Vec<OpCode> {
    for stmt in block {
        match stmt {
            Statement::SetV(name, value) => {
                let id = if let Some(id) = locals.iter().position(|l| l == name) {
                    id
                } else {
                    locals.push(name.to_owned());
                    locals.len() - 1
                };
                result.extend(compile_expr(class_tree, &locals, value));
                result.push(SetV(id));
            },
            Statement::SetF(obj, name, value) => {
                result.extend(compile_expr(class_tree, &locals, obj));
                result.extend(compile_expr(class_tree, &locals, value));
                result.push(SetF(name.to_owned()));
            },
            Statement::Call(obj, name, args) => {
                result.extend(compile_expr(class_tree, &locals, &*obj));
                result.extend(args.iter().flat_map(|a| compile_expr(class_tree, &locals, a)));
                result.push(Call(name.to_owned(), args.len()));
                result.push(Pop);
            },
            Statement::Return(value) => {
                result.extend(compile_expr(class_tree, &locals, value));
                result.push(Return);
            },
            Statement::If(condition, block) => {
                result.extend(compile_expr(class_tree, &locals, condition));
                let jump_index = result.len();
                result.push(Pop);
                compile_block(class_tree, result, locals, block);
                result[jump_index] = Jump(false, result.len())
            },
            Statement::While(condition, block) => {
                result.extend(compile_expr(class_tree, &locals, condition));
                let jump_index = result.len();
                result.push(Pop);
                compile_block(class_tree, result, locals, block);
                result.extend(compile_expr(class_tree, &locals, condition));
                result.push(Jump(true, jump_index + 1));
                result[jump_index] = Jump(false, result.len())
            },
        }
    }

    result.to_owned()
}

fn compile_method(class_tree: &ClassTree, method: &Method) -> CompiledMethod {
    if let Some(body) = &method.body {
        CompiledMethod {
            name: method.name.to_owned(),
            body: Some(compile_block(class_tree, &mut Vec::new(), &mut method.params.to_owned(), &body)),
            locals_size: method.params.len(),
        }
    } else {
        CompiledMethod {
            name: method.name.to_owned(),
            body: None,
            locals_size: 0,
        }
    }
}

pub fn compile(class_tree: &ClassTree) -> Vec<CompiledClass> {
    let mut result: Vec<CompiledClass> = Vec::with_capacity(class_tree.classes.len());
    
    for c in &class_tree.classes {
        let (inherited_fields, inherited_methods) = if let Some(pname) = c.parent.to_owned() {
            let parent = result[class_tree.map.get(&pname).unwrap().start].to_owned();
            (parent.fields, parent.methods)
        } else {
            (vec![], vec![])
        };
        result.push(CompiledClass {
            is_abstract: c.is_abstract,
            fields: inherited_fields.iter().filter(|f| !c.own_fields.contains(f)).chain(c.own_fields.iter()).map(String::to_owned).collect(),
            methods: inherited_methods.iter().filter(|m| !c.own_methods.iter().any(|mm| mm.name == m.name)).map(CompiledMethod::to_owned).chain(c.own_methods.iter().map(|m| compile_method(class_tree, m))).collect(),
        });
    }

    result
}
