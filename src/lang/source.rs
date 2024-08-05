use super::value;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SourceInfo {
    pub name: Rc<str>,
    pub lineno: usize,
    pub charno: usize,
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: Rc<str>,
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug)]
pub struct List {
    pub list: Rc<Vec<Value>>,
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(List),
}

impl Value {
    pub fn source_info(&self) -> &SourceInfo {
        match self {
            Value::Atom(atom) => &atom.source_info,
            Value::List(list) => &list.source_info,
        }
    }
}

impl std::ops::Drop for List {
    fn drop(&mut self) {
        let mut list: Rc<Vec<Value>> = Rc::new(Vec::new());
        std::mem::swap(&mut self.list, &mut list);

        let list = match Rc::into_inner(list) {
            None => return,
            Some(list) if list.is_empty() => return,
            Some(list) => list,
        };

        #[cfg(test)]
        thread_local! { static IN_CALL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

        #[cfg(test)]
        {
            if IN_CALL.get() {
                panic!("Recursion!");
            } else {
                IN_CALL.set(true);
            }
        }

        let mut to_drop: VecDeque<Vec<Value>> = VecDeque::new();
        to_drop.push_back(list);

        while let Some(mut l) = to_drop.pop_front() {
            while let Some(v) = l.pop() {
                if let Value::List(mut list) = v {
                    let mut tmp: Rc<Vec<Value>> = Rc::new(Vec::new());
                    std::mem::swap(&mut tmp, &mut list.list);
                    std::mem::drop(list);

                    if let Some(v) = Rc::into_inner(tmp) {
                        to_drop.push_back(v);
                    }
                }
            }
        }

        #[cfg(test)]
        IN_CALL.set(false);
    }
}

impl std::fmt::Display for SourceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.name, self.lineno, self.charno)
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}'", self.name)
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        if !self.list.is_empty() {
            write!(f, "{}", self.list[0])?;
            for v in &self.list[1..] {
                write!(f, " {}", v)?;
            }
        }
        write!(f, ")")
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Atom(atom) => write!(f, "{}", atom),
            Value::List(list) => write!(f, "{}", list),
        }
    }
}

impl From<&List> for value::List {
    fn from(src: &List) -> Self {
        let mut head = value::List::Empty;
        for v in src.list.iter().rev() {
            head = value::List::Head(Rc::new(value::ListHead {
                val: v.into(),
                tail: head,
            }))
        }

        head
    }
}
