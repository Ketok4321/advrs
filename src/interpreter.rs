use crate::class_table::*;
use crate::opcode::*;
use crate::opcode::OpCode::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Object {
    pub class: usize,
    pub pointer: usize,
}

impl Object {
    pub fn class_name(&self, class_table: &ClassTable) -> String {
        class_table.classes[self.class].name.to_owned()
    }
}

pub struct RunCtx {
    pub class_table: ClassTable,
    pub classes: Vec<CompiledClass>,
}

pub fn new(class: usize) -> Object { // TODO: Proper allocations
    Object {
        class,
        pointer: 0
    }
}

fn null(ctx: &RunCtx) -> Object {
    new(ctx.class_table.null.start)
}

fn bool(ctx: &RunCtx, b: bool) -> Object {
    if b {
        new(ctx.class_table.truth.start)
    } else {
        new(ctx.class_table.lie.start)
    }
}

pub fn run(ctx: &RunCtx, method: &CompiledMethod, this: Object, args: &Vec<Object>) -> Object {
    if let Some(ops) = &method.body {
        let mut stack = Vec::with_capacity(8); // TODO: Reuse between methods
        let mut vars = vec![new(0); method.locals_size];
        vars[0..args.len()].copy_from_slice(args);

        let mut i = 0;
        while i < ops.len() {
            match ops[i].to_owned() {
                New(class) => stack.push(new(class)),
                GetV(id) => stack.push(vars[id]), // TODO: Handle null
                This => stack.push(this),
                Call(name, argc) => {
                    let argv = stack.split_off(stack.len() - argc);
                    let obj = stack.pop().unwrap();
                    let method = ctx.classes[obj.class].methods.iter().find(|&m| m.name == name).unwrap();
                    
                    stack.push(run(ctx, method, obj, &argv));
                },
                Is(range) => {
                    let class = stack.pop().unwrap().class;
                    stack.push(bool(ctx, range.matches(class)));
                },
                Equals => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(bool(ctx, a == b));
                }
                SetV(id) => vars[id] = stack.pop().unwrap(),
                Return => {
                    assert!(stack.len() == 1);
                    return stack.pop().unwrap()
                },
                Jump(expected, location) => {
                    if !expected ^ (ctx.class_table.truth.matches(stack.pop().unwrap().class)) {
                        i = location;
                        continue;
                    }
                },
                Pop => _ = stack.pop().unwrap(),
                _ => panic!(),
            }
            i += 1;
        }
        assert!(stack.len() == 0);
        null(ctx)
    } else {
        panic!();
    }
}
