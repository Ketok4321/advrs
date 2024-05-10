use crate::class_tree::*;
use crate::opcode::*;
use crate::opcode::OpCode::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Object {
    pub class: usize,
    pub pointer: usize,
}

pub fn new(class: usize) -> Object { // TODO: Proper allocations
    Object {
        class,
        pointer: 0
    }
}

fn bool(class_tree: &ClassTree, b: bool) -> Object {
    if b {
        new(class_tree.truth.start)
    } else {
        new(class_tree.lie.start)
    }
}

pub fn run(class_tree: &ClassTree, classes: &Vec<CompiledClass>, method: &CompiledMethod, this: Object, args: &Vec<Object>) -> Object {
    let mut stack = Vec::with_capacity(8); // TODO: Alloc on stack
    let mut vars = [new(0); 16]; // TODO: Dynamic size
    vars[0..args.len()].copy_from_slice(args);
    
    if let Some(ops) = &method.body {
        let mut i = 0;
        while i < ops.len() {
            match ops[i].to_owned() {
                New(class) => stack.push(new(class)),
                GetV(id) => stack.push(vars[id]), // TODO: Handle null
                This => stack.push(this),
                Call(name, argc) => {
                    let argv = stack.split_off(stack.len() - argc);
                    let obj = stack.pop().unwrap();
                    let method = classes[obj.class].methods.iter().find(|&m| m.name == name).unwrap();
                    
                    stack.push(run(class_tree, classes, method, obj, &argv));
                },
                Is(range) => {
                    let class = stack.pop().unwrap().class;
                    stack.push(bool(class_tree, range.matches(class)));
                },
                Equals => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(bool(class_tree, a == b));
                }
                SetV(id) => vars[id] = stack.pop().unwrap(),
                Return => {
                    assert!(stack.len() == 1);
                    return stack.pop().unwrap()
                },
                Jump(expected, location) => {
                    if !expected ^ (class_tree.truth.matches(stack.pop().unwrap().class)) {
                        i = location;
                        continue;
                    }
                },
                Pop => _ = stack.pop().unwrap(),
                _ => panic!(),
            }
            i += 1;
        }
    } else {
        panic!();
    }

    assert!(stack.len() == 0);

    new(0)
}
