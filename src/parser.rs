use crate::syntax::*;
use crate::lexer::*;

use std::slice::Iter;
use std::iter::Peekable;

fn parse_list<T>(iter: &mut Peekable<Iter<Token>>, parser: fn(&mut Peekable<Iter<Token>>) -> T) -> Vec<T> {
    assert_eq!(iter.next(), Some(&Token::OpeningParens));
    let mut elements = Vec::new();
    if iter.peek() == Some(&&Token::ClosingParens) {
        _ = iter.next()
    } else {
        loop {
            elements.push(parser(iter));
            match iter.next() {
                Some(&Token::Comma) => (),
                Some(&Token::ClosingParens) => break,
                _ => panic!("Expected comma or closing parenthesis"),
            }
        }
    }

    elements
}

pub fn parse_expression(iter: &mut Peekable<Iter<Token>>) -> Expression {
    if let Some(ch) = iter.next() {
        match ch {
            Token::Identifier(id) => parse_expression_further(iter, Expression::Get(id.to_string())),
            Token::StringLiteral(str) => parse_expression_further(iter, Expression::String(str.to_string())),
            Token::OpeningParens => {
                let result = parse_expression(iter);
                assert_eq!(iter.next(), Some(&Token::ClosingParens));
                parse_expression_further(iter, result)
            }
            _ => panic!("Expected identifier, string literal or opening parenthesis"),
        }
    } else {
        panic!("Expected expression");
    }
}

fn parse_expression_further(iter: &mut Peekable<Iter<Token>>, expr: Expression) -> Expression {
    match iter.peek() {
        Some(Token::Dot) => {
            _ = iter.next();
            if let Some(Token::Identifier(next)) = iter.next() {
                if let Some(Token::OpeningParens) = iter.peek() {
                    let args = parse_list(iter, parse_expression);
                    parse_expression_further(iter, Expression::Call(Box::new(expr), next.to_string(), args))
                } else {
                    parse_expression_further(iter, Expression::GetF(Box::new(expr), next.to_string()))
                }
            } else {
                panic!("Expected identifier");
            }
        },
        Some(Token::Is) => {
            _ = iter.next();
            if let Some(Token::Identifier(next)) = iter.next() {
                parse_expression_further(iter, Expression::Is(Box::new(expr), next.to_string()))
            } else {
                panic!("Expected identifier");
            }
        },
        Some(Token::EqualsSign) => {
            _ = iter.next();
            Expression::Equals(Box::new(expr), Box::new(parse_expression(iter)))
        },
        _ => expr
    }
}

pub fn parse_statement(iter: &mut Peekable<Iter<Token>>) -> Statement {
    match iter.peek() {
        Some(Token::Return) => {
            _ = iter.next();
            Statement::Return(parse_expression(iter))
        },
        Some(Token::If) => {
            _ = iter.next();
            Statement::If(parse_expression(iter), parse_block(iter))
        },
        Some(Token::While) => {
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
    
    assert_eq!(iter.next(), Some(&Token::BlockStart));
    while iter.peek() != Some(&&Token::BlockEnd) {
        result.push(parse_statement(iter));
    }
    _ = iter.next();

    result
}

pub fn parse_class(iter: &mut Peekable<Iter<Token>>) -> Class {
    assert_eq!(iter.next(), Some(&Token::Class));
    if let Some(Token::Identifier(name)) = iter.next() {
        assert_eq!(iter.next(), Some(&Token::Extends));
        if let Some(Token::Identifier(parent)) = iter.next() {
            assert_eq!(iter.next(), Some(&Token::BlockStart));
            
            let mut fields = Vec::new();
            let mut methods = Vec::new();

            loop {
                match iter.next() {
                    Some(Token::Field) => {
                        if let Some(Token::Identifier(name)) = iter.next() {
                            fields.push(name.to_string());
                        } else {
                            panic!("Expected identifier");
                        }
                    },
                    Some(Token::Method) => {
                        if let Some(Token::Identifier(name)) = iter.next() {
                            methods.push(Method {
                                name: name.to_string(),
                                params: parse_list(iter, |iter| if let Some(Token::Identifier(name)) = iter.next() { name.to_string() } else { panic!("Expected identifier"); }),
                                body: if iter.peek() == Some(&&Token::BlockStart) { Some(parse_block(iter)) } else { None },
                            });
                        } else {
                            panic!("Expected identifier");
                        }
                    },
                    Some(Token::BlockEnd) => break,
                    _ => panic!("Expected 'field' or 'method'"),
                }
            }

            Class {
                name: name.to_string(),
                is_abstract: false, // TODO: Abstract classes
                parent: Some(parent.to_string()),
                own_fields: fields,
                own_methods: methods,
            }
        } else {
            panic!("Expected identifier");
        }
    } else {
        panic!("Expected identifier");
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
