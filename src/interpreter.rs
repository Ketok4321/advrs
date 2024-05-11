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

pub fn new(class: usize) -> Object { // TODO: Proper allocations
    Object {
        class,
        pointer: 0
    }
}

pub fn null(class_table : &ClassTable) -> Object {
    new(class_table.null.start)
}

fn bool(class_table: &ClassTable, b: bool) -> Object {
    if b {
        new(class_table.truth.start)
    } else {
        new(class_table.lie.start)
    }
}

pub fn run(class_table: &ClassTable, classes: &Vec<CompiledClass>, method: &CompiledMethod, this: Object, args: &Vec<Object>) -> Object {
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
                    let method = classes[obj.class].methods.iter().find(|&m| m.name == name).unwrap();
                    
                    stack.push(run(class_table, classes, method, obj, &argv));
                },
                Is(range) => {
                    let class = stack.pop().unwrap().class;
                    stack.push(bool(class_table, range.matches(class)));
                },
                Equals => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(bool(class_table, a == b));
                }
                SetV(id) => vars[id] = stack.pop().unwrap(),
                Return => {
                    assert!(stack.len() == 1);
                    return stack.pop().unwrap()
                },
                Jump(expected, location) => {
                    if !expected ^ (class_table.truth.matches(stack.pop().unwrap().class)) {
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
        null(class_table)
    } else {
        panic!();
    }
}
