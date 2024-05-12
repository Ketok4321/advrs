use crate::class_table::*;
use crate::opcode::*;
use crate::opcode::OpCode::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Object {
    pub class: usize,
    pub contents: *mut [Object],
}

impl Object {
    const TRUE_NULL: Object = Object { class: 0, contents: std::ptr::null_mut::<[Object;0]>() as *mut [Object]};
    
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

pub fn run(ctx: &RunCtx, method: &CompiledMethod, this: Object, args: &Vec<Object>) -> Object {
    if let Some(ops) = &method.body {
        let mut stack = Vec::with_capacity(8); // TODO: Reuse between methods
        let mut vars = vec![Object::TRUE_NULL; method.locals_size];
        vars[0..args.len()].copy_from_slice(args);

        let mut i = 0;
        while i < ops.len() {
            match ops[i].to_owned() {
                New(class) => stack.push(Object::new(ctx, class)),
                GetV(id) => {
                    assert_ne!(vars[id], Object::TRUE_NULL);
                    stack.push(vars[id]);
                },
                This => stack.push(this),
                GetF(name) => {
                    let obj = stack.pop().unwrap();
                    if let Some(index) = ctx.classes[obj.class].fields.iter().position(|f| **f == name) {
                        unsafe { stack.push((*obj.contents)[index]); }
                    } else {
                        panic!("No such field: {name}");
                    }
                },
                Call(name, argc) => {
                    let argv = stack.split_off(stack.len() - argc);
                    let obj = stack.pop().unwrap();
                    let method = ctx.classes[obj.class].methods.iter().find(|&m| m.name == name).unwrap();
                    
                    stack.push(run(ctx, method, obj, &argv));
                },
                Is(range) => {
                    let class = stack.pop().unwrap().class;
                    stack.push(Object::bool(ctx, range.matches(class)));
                },
                Equals => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(Object::bool(ctx, a == b));
                }
                SetV(id) => vars[id] = stack.pop().unwrap(),
                SetF(name) => {
                    let value = stack.pop().unwrap();
                    let obj = stack.pop().unwrap();

                    if let Some(index) = ctx.classes[obj.class].fields.iter().position(|f| **f == name) {
                        unsafe { (*obj.contents)[index] = value; }
                    } else {
                        panic!("No such field: {name}");
                    }
                },
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
        Object::null(ctx)
    } else {
        panic!("Attempted to run a method without a body");
    }
}
