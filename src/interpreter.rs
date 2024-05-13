use crate::class_table::*;
use crate::opcode::*;
use crate::opcode::OpCode::*;
use crate::gc::*;

#[derive(Clone, Copy, Debug, Eq)]
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
    
    pub fn class_name(&self, class_table: &ClassTable) -> String {
        class_table.classes[self.class].name.to_owned()
    }
    
    pub fn is(&self, range: &TypeRange) -> bool {
        range.matches(self.class)
    }
}

impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.contents.hash(state);
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(self.contents, other.contents)
    }
}

pub struct RunCtx {
    pub class_table: ClassTable,
    pub classes: Vec<CompiledClass>,
}

pub fn run(ctx: &RunCtx, gc: &mut GC, full_stack: &mut [Object], method: &CompiledMethod) -> Object {
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
            match ops[i].to_owned() {
                New(class) => push!(Object::new(ctx, gc, class)),
                GetV(id) => {
                    assert_ne!(vars[id], Object::TRUE_NULL);
                    push!(vars[id]);
                },
                This => push!(*this),
                GetF(name) => {
                    let obj = pop!();
                    if let Some(index) = ctx.classes[obj.class].fields.iter().position(|f| **f == name) {
                        unsafe { push!((*obj.contents)[index]); }
                    } else {
                        panic!("No such field: {name}");
                    }
                },
                Call(name, argc) => {
                    let obj_i = stack_pos - argc - 1;
                    let obj = stack[obj_i];
                    let method = ctx.classes[obj.class].methods.iter().find(|&m| m.name == name).unwrap();

                    stack_pos = obj_i;
                    push!(run(ctx, gc, &mut stack[obj_i..], method));
                },
                Is(range) => {
                    push!(Object::bool(ctx, gc, pop!().is(&range)));
                },
                Equals => {
                    let a = pop!();
                    let b = pop!();
                    push!(Object::bool(ctx, gc, a == b));
                }
                SetV(id) => vars[id] = pop!(),
                SetF(name) => {
                    let value = pop!();
                    let obj = pop!();

                    if let Some(index) = ctx.classes[obj.class].fields.iter().position(|f| **f == name) {
                        unsafe { (*obj.contents)[index] = value; }
                    } else {
                        panic!("No such field: {name}");
                    }
                },
                Return => {
                    assert_eq!(stack_pos, 1);
                    return pop!();
                },
                Jump(expected, location) => {
                    if !expected ^ (pop!().is(&ctx.class_table.truth)) {
                        i = location;
                        continue;
                    }
                },
                Pop => _ = pop!(),
                _ => panic!(),
            }
            i += 1;
        }
        assert_eq!(stack_pos, 0);
    } else {
        match method.name.as_str() {
            "write" if this.is(&ctx.class_table.output) => {
                assert_ne!(rest[0], Object::TRUE_NULL);
                println!("{}", rest[0].class_name(&ctx.class_table));
            },
            _ => panic!("Attempted to run a method without a body"),
        }
    }
    Object::null(ctx, gc)
}
