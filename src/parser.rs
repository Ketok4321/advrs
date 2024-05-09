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
        Option::Some(Token::Dot) => {
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
        Option::Some(Token::Is) => {
            _ = iter.next();
            if let Some(Token::Identifier(next)) = iter.next() {
                parse_expression_further(iter, Expression::Is(Box::new(expr), next.to_string()))
            } else {
                panic!("Expected identifier");
            }
        },
        Option::Some(Token::EqualsSign) => {
            _ = iter.next();
            Expression::Equals(Box::new(expr), Box::new(parse_expression(iter)))
        },
        _ => expr
    }
}
