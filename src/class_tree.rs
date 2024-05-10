use std::collections::HashMap;

use crate::syntax::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct TypeRange {
    pub start: usize,
    pub end: usize,
}

impl TypeRange {
    pub fn matches(&self, class: usize) -> bool {
        return class >= self.start && class < self.end;
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ClassTree {
    pub classes: Vec<Class>,
    pub map: HashMap<String, TypeRange>, // Start is inclusive, end is exclusive
    pub truth: TypeRange,
    pub lie: TypeRange,
}

impl ClassTree {
    pub fn create(input: &Vec<Class>) -> ClassTree {
        let mut classes = Vec::with_capacity(input.len());
        let mut map = HashMap::with_capacity(input.len());

        fn add_with_parent(input: &Vec<Class>, parent: Option<String>, classes: &mut Vec<Class>, map: &mut HashMap<String, TypeRange>) {
            for c in input.iter().filter(|c| c.parent == parent) {
                let start = classes.len();
                classes.push(c.to_owned());
                add_with_parent(input, Some(c.name.to_owned()), classes, map);
                let end = classes.len();
                
                map.insert(c.name.to_owned(), TypeRange { start, end });
            }
        }

        add_with_parent(input, None, &mut classes, &mut map);

        let truth = map.get("True").unwrap().to_owned();
        let lie = map.get("False").unwrap().to_owned();

        ClassTree {
            classes,
            map,
            truth,
            lie,
        }
    }
}
