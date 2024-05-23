use std::io::prelude::*;

use anyhow::{Result, Context, bail, ensure};

use crate::class_table::*;
use crate::opcode::*;
use crate::opcode::OpCode::*;
use crate::gc::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Object {
    pub class: usize,
    pub contents: *mut [Object],
}

impl Object {
    pub const TRUE_NULL: Self = Self { class: 0, contents: std::ptr::null_mut::<[Self;0]>() as *mut [Self]};

    pub fn new(ctx: &RunCtx, gc: &mut GC, class: usize) -> Self {
        let cclass = &ctx.classes[class];
        let len = cclass.fields.len();
        let contents = gc.alloc(len);

        for i in 0..len {
            unsafe { (*contents)[i] = Self::null(ctx, gc) };
        }
        
        Self {
            class,
            contents,
        }
    }

    pub fn new_r(ctx: &RunCtx, gc: &mut GC, range: TypeRange) -> Self {
        if range == TypeRange::EMPTY {
            Self::null(ctx, gc)
        } else {
            Self::new(ctx, gc, range.0)
        }
    }

    pub fn null(ctx: &RunCtx, gc: &mut GC) -> Self {
        Self::new_r(ctx, gc, ctx.class_table.null)
    }

    pub fn bool(ctx: &RunCtx, gc: &mut GC, b: bool) -> Self {
        if b {
            Self::new_r(ctx, gc, ctx.class_table.truth)
        } else {
            Self::new_r(ctx, gc, ctx.class_table.lie)
        }
    }
    
    pub fn class_name<'a>(&self, class_table: &'a ClassTable) -> &'a str {
        &class_table.classes[self.class].name
    }
    
    pub fn is(&self, range: &TypeRange) -> bool {
        range.matches(self.class)
    }
}

pub struct RunCtx {
    pub class_table: ClassTable,
    pub classes: Vec<CompiledClass>,
    pub entrypoint: Object,
}

impl RunCtx {
    pub fn new(gc: &mut GC, class_table: ClassTable, classes: Vec<CompiledClass>, entrypoint_class: usize) -> Self {
        let mut result = Self {
            class_table,
            classes,
            entrypoint: Object::TRUE_NULL,
        };
        result.entrypoint = Object::new(&result, gc, entrypoint_class);
        result
    }
}

pub struct IOManager {
    write_stack: String,
    read_stack: String,
}

impl IOManager {
    pub fn new() -> Self {
        Self {
            write_stack: String::new(),
            read_stack: String::new(),
        }
    }

    pub fn write_char(&mut self, c: char) {
        self.write_stack.push(c);
    }

    pub fn write_end(&mut self) {
        std::io::stdout().write_all(self.write_stack.as_bytes()).expect("Failed to write to stdout");
        self.write_stack.clear();
    }

    pub fn read_start(&mut self) {
        self.read_stack.clear();
        std::io::stdin().read_line(&mut self.read_stack).expect("Failed to read from stdin");
        self.read_stack = self.read_stack.chars().rev().collect();
    }

    pub fn read_char(&mut self) -> Option<char> {
        self.read_stack.pop()
    }
}

pub fn run(ctx: &RunCtx, gc: &mut GC, io: &mut IOManager, full_stack: &mut [Object], method: &CompiledMethod) -> Result<Object> {
    let (this, rest) = full_stack.split_first_mut().unwrap();
    if let Some(ops) = &method.body {
        let (vars, stack) = rest.split_at_mut(method.locals_size);

        let mut stack_pos = 0;

        macro_rules! push {
            ($value:expr) => {{
                stack[stack_pos] = $value;
                stack_pos += 1;
            }}
        }

        macro_rules! pop {
            () => {{
                stack_pos -= 1;
                let res = stack[stack_pos];
                stack[stack_pos] = Object::TRUE_NULL; // cuz garbage collector
                res
            }}
        }

        let mut i = 0;
        while i < ops.len() {
            match &ops[i] {
                New(class) => push!(Object::new(ctx, gc, *class)),
                GetV(id) => {
                    ensure!(vars[*id] != Object::TRUE_NULL, "Attempted to use a variable before its initialization");
                    push!(vars[*id]);
                },
                This => push!(*this),
                GetF(name) => {
                    let obj = pop!();
                    if let Some(index) = ctx.classes[obj.class].fields.iter().position(|f| f == name) {
                        unsafe { push!((*obj.contents)[index]); }
                    } else {
                        bail!("No such field: {name}");
                    }
                },
                GetFI(index) => {
                    let obj = pop!();
                    unsafe { push!((*obj.contents)[*index]); }
                },
                Call(name, argc) => {
                    let obj_i = stack_pos - argc - 1;
                    let obj = stack[obj_i];
                    let method = ctx.classes[obj.class].methods.iter().find(|m| m.name == *name).with_context(|| format!("Type '{}' doesn't define method '{}'", obj.class_name(&ctx.class_table), name))?;
                    ensure!(*argc == method.params_count, "Method '{}.{}' takes {} arguments, but {} were provided", obj.class_name(&ctx.class_table), name, method.params_count, argc);

                    stack_pos = obj_i;
                    push!(run(ctx, gc, io, &mut stack[obj_i..], method).with_context(|| format!("Failed to run method '{}.{}'", obj.class_name(&ctx.class_table), name))?);
                },
                Is(range) => {
                    push!(Object::bool(ctx, gc, pop!().is(&range)));
                },
                Equals => {
                    let a = pop!();
                    let b = pop!();
                    push!(Object::bool(ctx, gc, a == b));
                }
                SetV(id) => vars[*id] = pop!(),
                SetF(name) => {
                    let value = pop!();
                    let obj = pop!();

                    if let Some(index) = ctx.classes[obj.class].fields.iter().position(|f| f == name) {
                        unsafe { (*obj.contents)[index] = value; }
                    } else {
                        bail!("No such field: {name}");
                    }
                },
                SetFI(index) => {
                    let value = pop!();
                    let obj = pop!();

                    unsafe { (*obj.contents)[*index] = value; }
                },
                Return => {
                    assert_eq!(stack_pos, 1);
                    return Ok(pop!());
                },
                Jump(expected, location) => {
                    if !expected ^ (pop!().is(&ctx.class_table.truth)) {
                        i = *location;
                        continue;
                    }
                },
                Pop => _ = pop!(),
            }
            i += 1;
        }
        assert_eq!(stack_pos, 0);
    } else {
        if *this == ctx.entrypoint {
            match method.name.as_str() {
                "writeChar" => {
                    let class_name = rest[0].class_name(&ctx.class_table);
                    let char = class_name.strip_prefix('\'').unwrap().strip_suffix('\'').unwrap();
                    assert_eq!(char.len(), 1);
                    io.write_char(char.chars().nth(0).unwrap());
                },
                "endWrite" => {
                    io.write_end();
                },
                "startRead" => {
                    io.read_start();
                },
                "readChar" => {
                    if let Some(c) = io.read_char() {
                        if let Some(class) = ctx.class_table.map.get(&format!("'{c}'")) {
                            return Ok(Object::new_r(ctx, gc, *class));
                        } else {
                            return Ok(Object::null(ctx, gc));
                        }
                    } else {
                        return Ok(Object::null(ctx, gc));
                    }
                },
                _ => bail!("Attempted to run a method without a body on an entrypoint class"),
            }
        } else {
            bail!("Attempted to run a method without a body");
        }
    }
    Ok(Object::null(ctx, gc))
}
