use super::source;
use std::boxed::Box;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: Box<str>,
}

#[derive(Clone, Debug)]
pub enum List {
    Empty,
    Head(Rc<ListHead>),
}

#[derive(Debug)]
pub struct ListHead {
    pub val: Value,
    pub tail: List,
}

#[derive(Clone, Debug)]
pub enum ArgList {
    Vargs(Atom),
    Args(Box<[Atom]>),
}

#[derive(Clone, Debug)]
pub struct Fexpr {
    pub arg_list: ArgList,
    pub body: Rc<List>,
}

#[derive(Clone, Debug)]
pub enum Value {
    SourceAtom(source::Atom),
    SourceList(source::List),
    Atom(Atom),
    List(List),
    Fexpr(Fexpr),
}

impl Ord for Atom {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Atom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for Atom {}

impl Hash for Atom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<&source::Atom> for Atom {
    fn from(src: &source::Atom) -> Self {
        Self {
            name: src.name.clone(),
        }
    }
}

impl From<&source::Value> for Value {
    fn from(src: &source::Value) -> Self {
        match src {
            source::Value::Atom(a) => Value::SourceAtom(a.clone()),
            source::Value::List(l) => Value::SourceList(l.clone()),
        }
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}'", self.name)
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            List::Empty => write!(f, "()"),
            List::Head(head) => match head.tail {
                List::Empty => write!(f, "({})", head.val),
                List::Head(_) => write!(f, "({} ...)", head.val),
            },
        }
    }
}

impl std::fmt::Display for Fexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.arg_list {
            ArgList::Args(list) => {
                write!(f, "(fexpr! (")?;
                if !list.is_empty() {
                    write!(f, "{}", list[0])?;
                    for atom in &list[1..] {
                        write!(f, " {}", atom)?;
                    }
                }
                write!(f, ") {})", self.body)
            }
            ArgList::Vargs(atom) => write!(f, "(fexpr! {} {})", atom, self.body),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atom(atom) => write!(f, "{}", atom),
            Self::List(list) => write!(f, "{}", list),
            Self::SourceAtom(source_atom) => write!(f, "{}", source_atom),
            Self::SourceList(source_list) => write!(f, "{}", source_list),
            Self::Fexpr(fexpr) => write!(f, "{}", fexpr),
        }
    }
}
