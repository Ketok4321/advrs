use std::collections::HashMap;

use anyhow::{Result, Context, ensure};

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
}

impl ClassTable {
    pub fn create(input: &Vec<Class>) -> Result<ClassTable> {
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

        fn add_with_parent(input: &Vec<Class>, parent: Option<String>, classes: &mut Vec<Class>, map: &mut HashMap<String, TypeRange>, parent_map: &mut HashMap<Option<String>, Vec<&Class>>) {
            if let Some(pclasses) = parent_map.remove(&parent) {
                for c in pclasses {
                    let start = classes.len();
                    classes.push(c.to_owned());
                    add_with_parent(input, Some(c.name.to_owned()), classes, map, parent_map);
                    let end = classes.len();
                    
                    map.insert(c.name.to_owned(), TypeRange(start, end));
                }
            }
        }

        add_with_parent(input, None, &mut classes, &mut map, &mut parent_map);

        ensure!(parent_map.len() == 0, "Classes {:?} have invalid parents ({:?})", parent_map.values().flatten().map(|c| c.name.to_owned()).collect::<Vec<_>>(), parent_map.keys().map(Option::to_owned).map(Option::unwrap).collect::<Vec<_>>());

        let null = map.get("Null").unwrap().to_owned(); // This one will always unwrap
        let truth = map.get("True").unwrap_or(&TypeRange::EMPTY).to_owned();
        let lie = map.get("False").unwrap_or(&TypeRange::EMPTY).to_owned();

        Ok(ClassTable {
            classes,
            map,
            null,
            truth,
            lie,
        })
    }

    pub fn get_class_id(&self, name: &str) -> Result<usize> {
        Ok(self.map.get(name).with_context(|| format!("Couldn't find a class named {}", name))?.0)
    }

    pub fn get_class(&self, name: &str) -> Result<&Class> {
        self.get_class_id(name).map(|i| &self.classes[i])
    }
}
