use advrs::lexer::*;
use advrs::syntax::*;
use advrs::parser::*;
use advrs::class_table::*;
use advrs::opcode::*;
use advrs::interpreter::*;

const CODE: &str = r#"
class Program extends Object:
    method main()
end

class Input extends Object:
    field program    
    
    method read()
end

class Output extends Object:
    field program    
    
    method write(text)
end

class Boolean extends Object:
    method not()
    
    method and(b)
    
    method or(b)
end

class True extends Boolean:
    method not():
        return False
    end
    
    method and(b):
        return b
    end
    
    method or(b):
        return True
    end
end

class False extends Boolean:
    method not():
        return True
    end
    
    method and(b):
        return False
    end
    
    method or(b):
        return b
    end
end

class Number extends Object:
    method ++()
    
    method --()
    
    method isZero()
    
    method toString()
    
    method +(n):
        if n.isZero():
            return this
        end
        return (this.++()).+(n.--())
    end
    
    method -(n):
        if n.isZero():
            return this
        end
        return (this.--()).-(n.--())
    end
    
    method *(n):
        result = this.-(this)
        while (n.isZero()).not():
            result = result.+(this)
            n = n.--()
        end
        return result
    end
    
    method /(n):
        result = this.-(this)
        a = this
        while (a.isZero()).not():
            a = a.-(n)
            result = result.++()
        end
        return result
    end
    
    method equals(n):
        return this.equalsStr(n.toString())
    end
    
    method equalsStr(str):
        return (this.toString()).equals(str)
    end
    
    method greaterThan(n):
        if n.isZero():
            if this.isZero():
                return False
            end
            return True
        end
        if this.isZero():
            return False
        end
        return (this.--()).greaterThan(n.--())
    end
    
    method lesserThan(n):
        if n.isZero():
            if this.isZero():
                return False
            end
            return False
        end
        if this.isZero():
            return True
        end
        return (this.--()).lesserThan(n.--())
    end
end

class ClassNumber extends Number:
    method isZero():
        return False
    end
end

class 0 extends ClassNumber:
    method ++():
        return 1
    end
    
    method isZero():
        return True
    end
end

class 1 extends ClassNumber:
    method ++():
        return 2
    end
    
    method --():
        return 0
    end
end

class 2 extends ClassNumber:
    method ++():
        return 3
    end
    
    method --():
        return 1
    end
end

class 3 extends ClassNumber:
    method ++():
        return 4
    end
    
    method --():
        return 2
    end
end

class 4 extends ClassNumber:
    method ++():
        return 5
    end
    
    method --():
        return 3
    end
end

class 5 extends ClassNumber:
    method --():
        return 4
    end
end

class LinkedList extends Object:
    field _first
    field _last

    method push(value):
        cell = _LinkedList_Cell
        cell.value = value
        cell.prev = this._last
        
        if (this._last) is _LinkedList_Cell: # (non-null check)
            this._last.next = cell
        end

        this._last = cell
        if (this._first) is Null:
            this._first = cell
        end
    end

    method pop():
        old = this._last
        this._last = old.prev

        if (this._last) is _LinkedList_Cell:
            this._last.next = Null
        end

        if (this._first) = old:
            this._first = Null
        end

        return old.value
    end

    method pushStart(value):
        cell = _LinkedList_Cell
        cell.value = value
        cell.next = this._first
        
        if (this._first) is _LinkedList_Cell: # (non-null check)
            this._first.prev = cell
        end

        this._first = cell
        if (this._last) is Null:
            this._last = cell
        end
    end

    method popStart():
        old = this._first
        this._first = old.next

        if (this._first) is _LinkedList_Cell:
            this._first.prev = Null
        end

        if (this._last) = old:
            this._last = Null
        end

        return old.value
    end

    method first():
        return this._first.value
    end

    method last():
        return this._last.value
    end
end

class _LinkedList_Cell extends Object:
    field value
    field prev
    field next
end

class Test extends Program:
    field list

    method main():
        this.list = LinkedList
        this.list.push(True)
        this.list.push(False)
        this.list.push(this.list.first())
        this.list.push(2.*(2))

        return this.list.pop()
    end
end
"#;

fn main() {
    let mut classes = vec![
        Class {
            name: "Object".to_string(),
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
        Class {
            name: "String".to_string(),
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
        Class {
            name: "Null".to_string(),
            parent: None,
            own_fields: vec![],
            own_methods: vec![]
        },
    ];
    classes.extend(parse(tokenize(CODE)));
    let table = ClassTable::create(&classes);
    let compiled = compile(&table);
    let entrypoint = table.program.start + 1;
    let ctx = RunCtx {
        class_table: table,
        classes: compiled,
    };
    let res = run(&ctx, ctx.classes[entrypoint].methods.iter().find(|m| m.name == "main").unwrap(), Object::new(&ctx, entrypoint), &vec![]);

    println!("{}", res.class_name(&ctx.class_table));
}
