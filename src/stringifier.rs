use crate::syntax::*;
use crate::syntax::Expression::*;

struct CodeBuilder {
    code: String,
    tab_index: usize
}

impl CodeBuilder {
    pub fn new() -> Self {
        Self {
            code: String::new(),
            tab_index: 0
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            code: String::with_capacity(cap),
            tab_index: 0
        }
    }

    pub fn line<S: AsRef<str>>(&mut self, s: S) -> &mut Self {
        for _ in 0..self.tab_index {
            self.code.push_str("    ");
        }
        self.code.push_str(s.as_ref());
        self.code.push_str("\n");
        self
    }

    pub fn newline(&mut self) -> &mut Self {
        self.code.push_str("\n");
        self
    }

    pub fn tab(&mut self) -> &mut Self {
        self.tab_index += 1;
        self
    }

    pub fn untab(&mut self) -> &mut Self {
        self.tab_index -= 1;
        self
    }

    pub fn into_string(self) -> String {
        self.code
    }

    pub fn as_str(&self) -> &str {
        &self.code
    }
}

fn stringify_list<T>(list: &Vec<T>, stringifier: fn(&T) -> String) -> String {
    format!("({})", list.iter().map(stringifier).collect::<Vec<String>>().join(", "))
}

fn stringify_expression(expr: &Expression) -> String {
    match expr {
        Get(name) => name.to_owned(),
        GetF(obj, name) => stringify_expression(obj) + "." + name,
        Call(obj, name, args) => stringify_expression(obj) + "." + name + stringify_list(args, stringify_expression).as_str(),
        Is(obj, class) => format!("{} is {}", stringify_expression(obj), class),
        Equals(obj1, obj2) => format!("{} = {}", stringify_expression(obj1), stringify_expression(obj2)),
    }
}

fn stringify_statement(bd: &mut CodeBuilder, stmt: &Statement) {
    match stmt {
        Statement::Return(expr) => {
            bd.line(format!("return {}", stringify_expression(expr)));
        },
        Statement::If(cond, block) => {
            bd.line(format!("if {}:", stringify_expression(cond)));
            stringify_block(bd, block);
        },
        Statement::While(cond, block) => {
            bd.line(format!("while {}:", stringify_expression(cond)));
            stringify_block(bd, block);
        },
        Statement::SetV(var, val) => {
            bd.line(format!("{} = {}", var, stringify_expression(val)));
        },
        Statement::SetF(obj, field, val) => {
            bd.line(format!("{}.{} = {}", stringify_expression(obj), field, stringify_expression(val)));
        },
        Statement::Call(obj, method, args) => {
            bd.line(format!("{}.{}{}", stringify_expression(obj), method, stringify_list(args, stringify_expression)));
        }
    }
}

fn stringify_block(bd: &mut CodeBuilder, stmts: &[Statement]) {
    bd.tab();

    for s in stmts {
        stringify_statement(bd, s);
    }

    bd.untab().line("end");
}

fn stringify_class(bd: &mut CodeBuilder, class: &Class) {
    if let Some(p) = &class.parent {
        bd.line(format!("class {} extends {}:", class.name, p));
    } else {
        bd.line(format!("class {}:", class.name));
    }

    bd.tab();

    for f in &class.own_fields {
        bd.line(format!("field {}", f));
    }

    for m in &class.own_methods {
        bd.line(format!("method {}{}{}", m.name, stringify_list(&m.params, |s| s.to_string()), if m.body.is_some() { ":" } else { "" }));

        if let Some(b) = &m.body {
            stringify_block(bd, &b);
        }
    }

    bd.untab().line("end");
}

fn stringify_metadata(bd: &mut CodeBuilder, metadata: &Metadata) {
    bd.line(format!("target: '{}'", metadata.target));
    bd.newline();
    for import in &metadata.dependencies {
        bd.line(format!("import: '{}'", import));
    }
    bd.newline();
    for entry in &metadata.entrypoints {
        bd.line(format!("entrypoint: '{}'", entry));
    }
    bd.newline();
}

pub fn stringify(metadata: &Metadata, classes: &[Class]) -> String {
    let mut bd = CodeBuilder::new();

    stringify_metadata(&mut bd, metadata);

    for c in classes {
        stringify_class(&mut bd, c);
    }

    bd.into_string()
}
