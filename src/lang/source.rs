use super::value;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SourceInfo {
    pub name: Rc<str>,
    pub lineno: usize,
    pub charno: usize,
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: Rc<str>, // Might use Rc?
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug)]
pub struct List {
    pub list: Rc<[Value]>, // Might use Rc?
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(List),
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
