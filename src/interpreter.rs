use crate::class_table::*;
use crate::opcode::*;
use crate::opcode::OpCode::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Object {
    pub class: usize,
    pub contents: *mut [Object],
}

impl Object {
    pub const TRUE_NULL: Object = Object { class: 0, contents: std::ptr::null_mut::<[Object;0]>() as *mut [Object]};
    
    pub fn class_name(&self, class_table: &ClassTable) -> String {
        class_table.classes[self.class].name.to_owned()
    }

    pub fn new(ctx: &RunCtx, class: usize) -> Object {
        unsafe {
            let cclass = &ctx.classes[class];
            let len = cclass.fields.len();
            let layout = std::alloc::Layout::array::<Object>(len).expect("Invalid layout :<");
            let contents = std::ptr::slice_from_raw_parts(std::alloc::alloc(layout), len) as *mut [Object];

            for i in 0..len {
                (*contents)[i] = Object::new(ctx, ctx.class_table.null.start);
            }
            
            Object {
                class,
                contents,
            }
        }
    }

    pub fn null(ctx: &RunCtx) -> Object {
        Object::new(ctx, ctx.class_table.null.start)
    }

    pub fn bool(ctx: &RunCtx, b: bool) -> Object {
        if b {
            Object::new(ctx, ctx.class_table.truth.start)
        } else {
            Object::new(ctx, ctx.class_table.lie.start)
        }
    }
}

pub struct RunCtx {
    pub class_table: ClassTable,
    pub classes: Vec<CompiledClass>,
}

pub fn run(ctx: &RunCtx, full_stack: &mut [Object], method: &CompiledMethod) -> Object {
    if let Some(ops) = &method.body {
        let (this, rest) = full_stack.split_first_mut().unwrap();
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
                stack[stack_pos]
            }}
        }

        let mut i = 0;
        while i < ops.len() {
            match ops[i].to_owned() {
                New(class) => push!(Object::new(ctx, class)),
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
                    push!(run(ctx, &mut stack[obj_i..], method));
                },
                Is(range) => {
                    let class = pop!().class;
                    push!(Object::bool(ctx, range.matches(class)));
                },
                Equals => {
                    let a = pop!();
                    let b = pop!();
                    push!(Object::bool(ctx, a == b));
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
                    if !expected ^ (ctx.class_table.truth.matches(pop!().class)) {
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
        Object::null(ctx)
    } else {
        panic!("Attempted to run a method without a body");
    }
}
