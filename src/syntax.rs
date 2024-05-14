pub const CURRENT_VERSION: &str = "indev";

#[derive(PartialEq, Clone, Debug)]
pub enum Expression {
    Get(String),
    GetF(Box<Expression>, String),
    Call(Box<Expression>, String, Vec<Expression>),
    Is(Box<Expression>, String),
    Equals(Box<Expression>, Box<Expression>),
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

#[derive(PartialEq, Clone, Debug)]
pub struct Metadata {
    pub target: String,
    pub dependencies: Vec<String>,
    pub entrypoint: Option<String>,
}

impl Metadata {
    pub fn default() -> Self {
        Self {
            target: CURRENT_VERSION.to_string(),
            dependencies: vec![],
            entrypoint: None,
        }
    }
}
