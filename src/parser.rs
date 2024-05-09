use crate::syntax::*;
use crate::lexer::*;

use std::slice::Iter;
use std::iter::Peekable;

pub fn parse_expression(iter: &mut Peekable<Iter<Token>>) -> Expression {
    if let Some(ch) = iter.next() {
        match ch {
            Token::Identifier(id) => parse_expression_further(iter, Expression::Get(id.to_string())),
            Token::String(str) => parse_expression_further(iter, Expression::String(str.to_string())),
            Token::OpeningParenthesis => {
                let result = parse_expression(iter);
                assert_eq!(iter.next(), Some(&Token::ClosingParenthesis));
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
                if let Some(Token::OpeningParenthesis) = iter.peek() {
                    _ = iter.next();
                    let mut args = Vec::new();
                    if iter.peek() == Some(&&Token::ClosingParenthesis) {
                        _ = iter.next()
                    } else {
                        loop {
                            args.push(parse_expression(iter));
                            match iter.next() {
                                Some(&Token::Comma) => (),
                                Some(&Token::ClosingParenthesis) => break,
                                _ => panic!("Expected comma or closing parenthesis"),
                            }
                        }
                    }
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
