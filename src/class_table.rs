use std::collections::HashMap;

use crate::syntax::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct TypeRange(pub usize, pub usize);

impl TypeRange {
    pub const EMPTY: Self = Self(0, 0);

    pub fn matches(&self, class: usize) -> bool {
        return class >= self.0 && class < self.1;
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ClassTable {
    pub classes: Vec<Class>,
    pub map: HashMap<String, TypeRange>, // Start is inclusive, end is exclusive
    pub null: TypeRange,
    pub truth: TypeRange,
    pub lie: TypeRange,
    pub program: TypeRange,
    pub input: TypeRange,
    pub output: TypeRange,
}

impl ClassTable {
    pub fn create(input: &Vec<Class>) -> ClassTable {
        let mut classes = Vec::with_capacity(input.len());
        let mut map = HashMap::with_capacity(input.len());

        let mut parent_map: HashMap<Option<String>, Vec<&Class>> = HashMap::new();

        for c in input {
            if let Some(vec) = parent_map.get_mut(&c.parent) {
                vec.push(c);
            } else {
                parent_map.insert(c.parent.to_owned(), vec![c]);
            }
        }

        fn add_with_parent(input: &Vec<Class>, parent: Option<String>, classes: &mut Vec<Class>, map: &mut HashMap<String, TypeRange>, parent_map: &HashMap<Option<String>, Vec<&Class>>) {
            if let Some(pclasses) = parent_map.get(&parent) {
                for c in pclasses {
                    let start = classes.len();
                    classes.push(c.to_owned().to_owned());
                    add_with_parent(input, Some(c.name.to_owned()), classes, map, parent_map);
                    let end = classes.len();
                    
                    map.insert(c.name.to_owned(), TypeRange(start, end));
                }
            }
        }

        add_with_parent(input, None, &mut classes, &mut map, &parent_map);

        let null = map.get("Null").unwrap().to_owned(); // This one will always unwrap
        let truth = map.get("True").unwrap_or(&TypeRange::EMPTY).to_owned();
        let lie = map.get("False").unwrap_or(&TypeRange::EMPTY).to_owned();
        let program = map.get("Program").unwrap_or(&TypeRange::EMPTY).to_owned();
        let input = map.get("Input").unwrap_or(&TypeRange::EMPTY).to_owned();
        let output = map.get("Output").unwrap_or(&TypeRange::EMPTY).to_owned();

        ClassTable {
            classes,
            map,
            null,
            truth,
            lie,
            program,
            input,
            output
        }
    }

    pub fn get_class(&self, name: &String) -> Option<&Class> {
        if let Some(range) = self.map.get(name) {
            Some(&self.classes[range.0])
        } else {
            None
        }
    }
}
