use std::slice::Iter;
use std::iter::Peekable;

use anyhow::{Result, bail, ensure};

use crate::syntax::*;
use crate::lexer::*;

macro_rules! pmatch {
    ($ctx:expr, $( $kind:ident => $kind_expr:expr ),* $(,)?) => {
        match $ctx.iter.next() {
            $( Some(Token { kind: TokenKind::$kind, .. }) => $kind_expr, )*
            None => bail!("{}: Expected a token, found eof", $ctx.file_name),
            Some(Token { line, column, kind }) => {
                bail!("{}:{}:{}: Expected one of: {}; Got: {:?}", $ctx.file_name, line, column, stringify!($($kind),*), kind);
            },
        }
    };
    ($ctx:expr, Identifier($name:ident) => $ident_expr:expr, $( $kind:ident => $kind_expr:expr ),* $(,)?) => {
        match $ctx.iter.next() {
            Some(Token { kind: TokenKind::Identifier($name), .. }) => $ident_expr,
            $( Some(Token { kind: TokenKind::$kind, .. }) => $kind_expr, )*
            None => bail!("{}: Expected a token, found eof", $ctx.file_name),
            Some(Token { line, column, kind }) => {
                bail!("{}:{}:{}: Expected one of: Indentifier, {}; Got: {:?}", $ctx.file_name, line, column, stringify!($($kind),*), kind);
            },
        }
    };
}

macro_rules! pmatch_maybe {
    ($expr:expr, $( $pat:pat => $expr2:expr ),* $(,)?) => {
        match $expr.map(|t| &t.kind) {
            $( $pat => $expr2, )*
        }
    };
}

macro_rules! expect {
    ($ctx:expr, $pat:ident) => {
        pmatch!($ctx,
            $pat => (),
        );
    }
}

macro_rules! expect_identifier {
    ($ctx:expr) => {
        pmatch!($ctx,
            Identifier(name) => name,
        )
    }
}

macro_rules! is {
    ($token:expr, $kind:ident) => {
        matches!($token, Some(Token { kind: TokenKind::$kind, .. }))
    }
}

struct ParseCtx<'a> {
    pub iter: Peekable<Iter<'a, Token>>,
    pub file_name: &'a str,
}

fn parse_list<T>(ctx: &mut ParseCtx, parser: fn(&mut ParseCtx) -> Result<T>) -> Result<Vec<T>> {
    expect!(ctx, OpeningParens);
    let mut elements = Vec::new();
    if ctx.iter.next_if(|t| t.kind == TokenKind::ClosingParens) == None {
        loop {
            elements.push(parser(ctx)?);
            pmatch!(ctx,
                Comma => (),
                ClosingParens => break,
            );
        }
    }

    Ok(elements)
}

fn parse_expression(ctx: &mut ParseCtx) -> Result<Expression> {
    pmatch!(ctx,
        Identifier(name) => parse_expression_further(ctx, Expression::Get(name.to_owned())),
        OpeningParens => {
            let result = parse_expression(ctx)?;
            expect!(ctx, ClosingParens);
            parse_expression_further(ctx, result)
        },
    )
}

fn parse_expression_further(ctx: &mut ParseCtx, expr: Expression) -> Result<Expression> {
    pmatch_maybe!(ctx.iter.peek(), 
        Some(TokenKind::Dot) => {
            ctx.iter.next();
            let name = expect_identifier!(ctx);
            if let Some(TokenKind::OpeningParens) = ctx.iter.peek().map(|t| &t.kind) {
                let args = parse_list(ctx, parse_expression)?;
                parse_expression_further(ctx, Expression::Call(Box::new(expr), name.to_owned(), args))
            } else {
                parse_expression_further(ctx, Expression::GetF(Box::new(expr), name.to_owned()))
            }
        },
        Some(TokenKind::Is) => {
            ctx.iter.next();
            let name = expect_identifier!(ctx);
            parse_expression_further(ctx, Expression::Is(Box::new(expr), name.to_owned()))
        },
        Some(TokenKind::EqualsSign) => {
            ctx.iter.next();
            Ok(Expression::Equals(Box::new(expr), Box::new(parse_expression(ctx)?)))
        },
        _ => Ok(expr)
    )
}

fn parse_statement(ctx: &mut ParseCtx) -> Result<Statement> {
    Ok(pmatch_maybe!(ctx.iter.peek(),
        Some(TokenKind::Return) => {
            ctx.iter.next();
            Statement::Return(parse_expression(ctx)?)
        },
        Some(TokenKind::If) => {
            ctx.iter.next();
            Statement::If(parse_expression(ctx)?, parse_block(ctx)?)
        },
        Some(TokenKind::While) => {
            _ = ctx.iter.next();
            Statement::While(parse_expression(ctx)?, parse_block(ctx)?)
        },
        _ => {
            let expr = parse_expression(ctx)?;
            match expr {
                Expression::Equals(a, b) => match *a {
                    Expression::Get(var) => Statement::SetV(var, *b),
                    Expression::GetF(obj, field) => Statement::SetF(*obj, field, *b),
                    _ => bail!("You can't just set a random expression lol"),
                },
                Expression::Call(obj, method, args) => Statement::Call(*obj, method, args),
                _ => bail!("Expected a statement, got an expression instead"),
            }
        }
    ))
}

fn parse_block(ctx: &mut ParseCtx) -> Result<Vec<Statement>> {
    let mut result = Vec::new();
    
    expect!(ctx, BlockStart);
    while ctx.iter.next_if(|t| t.kind == TokenKind::BlockEnd) == None {
        result.push(parse_statement(ctx)?);
    }

    Ok(result)
}

fn parse_class(ctx: &mut ParseCtx) -> Result<Class> {
    expect!(ctx, Class);
    let name = expect_identifier!(ctx);
    expect!(ctx, Extends);
    let parent = expect_identifier!(ctx);
    expect!(ctx, BlockStart);
    
    let mut fields = Vec::new();
    let mut methods = Vec::new();

    loop {
        pmatch!(ctx,
            Field => {
                let name = expect_identifier!(ctx);
                fields.push(name.to_owned());
            },
            Method => {
                let name = expect_identifier!(ctx);
                methods.push(Method {
                    name: name.to_owned(),
                    params: parse_list(ctx, |ctx| Ok(expect_identifier!(ctx).to_owned()))?,
                    body: if is!(ctx.iter.peek(), BlockStart) {
                        Some(parse_block(ctx)?)
                    } else {
                        None
                    },
                });
            },
            BlockEnd => break,
        )
    }

    Ok(Class {
        name: name.to_owned(),
        parent: Some(parent.to_owned()),
        own_fields: fields,
        own_methods: methods,
    })
}

fn parse_metadata(ctx: &mut ParseCtx) -> Result<Metadata> {
    let mut result = Metadata::default();

    loop {
        pmatch_maybe!(ctx.iter.peek(),
            Some(TokenKind::Identifier(name)) => {
                ctx.iter.next();
                expect!(ctx, BlockStart);
                match name.as_str() {
                    "target" => result.target = expect_identifier!(ctx).to_owned(),
                    "import" => result.dependencies = parse_list(ctx, |ctx| Ok(expect_identifier!(ctx).to_owned()))?,
                    "entrypoint" => result.entrypoint = Some(expect_identifier!(ctx).to_owned()),
                    x => bail!("'{x}' is not a valid metadata entry"),
                }
            },
            _ => return Ok(result)
        )
    }
}

pub fn parse(file_name: &str, tokens: Vec<Token>) -> Result<(Metadata, Vec<Class>)> {
    let mut ctx = ParseCtx {
        iter: tokens.iter().peekable(),
        file_name
    };

    let metadata = parse_metadata(&mut ctx)?;
    ensure!(metadata.target == CURRENT_VERSION, "Incompatible version! (program targets '{}', running '{}')", metadata.target, CURRENT_VERSION);
    let mut classes = Vec::new();

    while ctx.iter.peek() != None {
        classes.push(parse_class(&mut ctx)?);
    }

    Ok((metadata, classes))
}
