#[derive(PartialEq, Clone, Debug)]
pub enum Expression {
    Get(String),
    GetF(Box<Expression>, String),
    Call(Box<Expression>, String, Vec<Expression>),
    Is(Box<Expression>, String),
    Equals(Box<Expression>, Box<Expression>),
    String(String),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Statement {
    SetV(String, Expression),
    SetF(Expression, String, Expression),
    Call(Expression, String, Vec<Expression>),
    Return(Expression),
    If(Expression, Vec<Statement>),
    While(Expression, Vec<Statement>),
}

#[derive(PartialEq, Clone, Debug)]
pub struct Method {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Vec<Statement>>
}

#[derive(PartialEq, Clone, Debug)]
pub struct Class {
    pub name: String,
    pub parent: Option<String>,
    pub own_fields: Vec<String>,
    pub own_methods: Vec<Method>,
}
