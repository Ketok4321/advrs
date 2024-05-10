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
    pub body: Option<Vec<OpCode>>
}

#[derive(PartialEq, Clone, Debug)]
pub struct CompiledClass {
    pub name: String,
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

pub fn compile_method(class_tree: &ClassTree, method: &Method) -> CompiledMethod {
    CompiledMethod {
        name: method.name.to_owned(),
        body: if let Some(body) = &method.body { Some(compile_block(class_tree, &mut Vec::new(), &mut method.params.to_owned(), &body)) } else { None },
    }
}

pub fn compile_class(class_tree: &ClassTree, class: &Class) -> CompiledClass {
    CompiledClass {
        name: class.name.to_owned(),
        is_abstract: class.is_abstract,
        fields: class.own_fields.to_owned(), // TODO: Inheritance
        methods: class.own_methods.iter().map(|m| compile_method(class_tree, m)).collect(),
    }
}
