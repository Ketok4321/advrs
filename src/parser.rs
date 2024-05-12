use crate::syntax::*;
use crate::lexer::*;
use crate::lexer::TokenKind as TK;

use std::slice::Iter;
use std::iter::Peekable;

macro_rules! pmatch {
    ($expr:expr, $( $pat:pat => $expr2:expr ),* $(,)?) => {
        match $expr {
            $( Some(Token { kind: $pat, .. }) => $expr2, )*
            None => panic!("Expected a token, found eof"),
            Some(Token { line, column, kind }) => {
                panic!("Line {}, column {}:\nExpected one of: {}\nGot: {:?}", line, column, stringify!($($pat),*), kind);
            }
        }
    };
}

macro_rules! require {
    ($token:expr, $pat:pat) => {
        pmatch!($token,
            $pat => (),
        );
    }
}

macro_rules! require_identifier {
    ($token:expr) => {
        pmatch!($token,
            TK::Identifier(name) => name,
        )
    }
}

macro_rules! is {
    ($token:expr, $kind:pat) => {
        matches!($token, Some(Token { kind: $kind, .. }))
    }
}

fn parse_list<T>(iter: &mut Peekable<Iter<Token>>, parser: fn(&mut Peekable<Iter<Token>>) -> T) -> Vec<T> {
    require!(iter.next(), TK::OpeningParens);
    let mut elements = Vec::new();
    if iter.next_if(|t| t.kind == TK::ClosingParens) == None {
        loop {
            elements.push(parser(iter));
            pmatch!(iter.next(),
                TK::Comma => (),
                TK::ClosingParens => break,
            );
        }
    }

    elements
}

pub fn parse_expression(iter: &mut Peekable<Iter<Token>>) -> Expression {
    pmatch!(iter.next(),
        TK::Identifier(id) => parse_expression_further(iter, Expression::Get(id.to_owned())),
        TK::StringLiteral(str) => parse_expression_further(iter, Expression::String(str.to_owned())),
        TK::OpeningParens => {
            let result = parse_expression(iter);
            require!(iter.next(), TK::ClosingParens);
            parse_expression_further(iter, result)
        },
    )
}

fn parse_expression_further(iter: &mut Peekable<Iter<Token>>, expr: Expression) -> Expression {
    match iter.peek().map(|t| &t.kind) {
        Some(TK::Dot) => {
            _ = iter.next();
            let name = require_identifier!(iter.next());
            if let Some(TK::OpeningParens) = iter.peek().map(|t| &t.kind) {
                let args = parse_list(iter, parse_expression);
                parse_expression_further(iter, Expression::Call(Box::new(expr), name.to_owned(), args))
            } else {
                parse_expression_further(iter, Expression::GetF(Box::new(expr), name.to_owned()))
            }
        },
        Some(TK::Is) => {
            _ = iter.next();
            let name = require_identifier!(iter.next());
            parse_expression_further(iter, Expression::Is(Box::new(expr), name.to_owned()))
        },
        Some(TK::EqualsSign) => {
            _ = iter.next();
            Expression::Equals(Box::new(expr), Box::new(parse_expression(iter)))
        },
        _ => expr
    }
}

pub fn parse_statement(iter: &mut Peekable<Iter<Token>>) -> Statement {
    match iter.peek().map(|t| &t.kind) {
        Some(TK::Return) => {
            _ = iter.next();
            Statement::Return(parse_expression(iter))
        },
        Some(TK::If) => {
            _ = iter.next();
            Statement::If(parse_expression(iter), parse_block(iter))
        },
        Some(TK::While) => {
            _ = iter.next();
            Statement::While(parse_expression(iter), parse_block(iter))
        },
        _ => {
            let expr = parse_expression(iter);
            match expr {
                Expression::Equals(a, b) => match *a {
                    Expression::Get(var) => Statement::SetV(var, *b),
                    Expression::GetF(obj, field) => Statement::SetF(*obj, field, *b),
                    _ => panic!("You can't just set a random expression lol"),
                },
                Expression::Call(obj, method, args) => Statement::Call(*obj, method, args),
                _ => panic!("Expected a statement, got an expression instead"),
            }
        }
    }
}

pub fn parse_block(iter: &mut Peekable<Iter<Token>>) -> Vec<Statement> {
    let mut result = Vec::new();
    
    require!(iter.next(), TK::BlockStart);
    while iter.next_if(|t| t.kind == TK::BlockEnd) == None {
        result.push(parse_statement(iter));
    }

    result
}

pub fn parse_class(iter: &mut Peekable<Iter<Token>>) -> Class {
    require!(iter.next(), TK::Class);
    let name = require_identifier!(iter.next());
    require!(iter.next(), TK::Extends);
    let parent = require_identifier!(iter.next());
    require!(iter.next(), TK::BlockStart);
    
    let mut fields = Vec::new();
    let mut methods = Vec::new();

    loop {
        pmatch!(iter.next(),
            TK::Field => {
                let name = require_identifier!(iter.next());
                fields.push(name.to_owned());
            },
            TK::Method => {
                let name = require_identifier!(iter.next());
                methods.push(Method {
                    name: name.to_owned(),
                    params: parse_list(iter, |iter| require_identifier!(iter.next()).to_owned()),
                    body: if is!(iter.peek(), TK::BlockStart) {
                        Some(parse_block(iter))
                    } else {
                        None
                    },
                });
            },
            TK::BlockEnd => break,
        )
    }

    Class {
        name: name.to_owned(),
        parent: Some(parent.to_owned()),
        own_fields: fields,
        own_methods: methods,
    }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Class> {
    let mut iter = tokens.iter().peekable();

    let mut classes = Vec::new();

    while iter.peek() != None {
        classes.push(parse_class(&mut iter));
    }

    classes
}
