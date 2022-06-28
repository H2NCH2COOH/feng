use super::source;
use std::boxed::Box;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: Box<str>,
}

#[derive(Debug)]
pub enum List {
    EmptyList {},
    Head { head: Value, tail: Rc<List> },
}

#[derive(Clone, Debug)]
pub enum ArgList {
    Vargs(Atom),
    Args(Box<[Atom]>),
}

#[derive(Clone, Debug)]
pub struct Fexpr {
    arg_list: ArgList,
    body: Rc<List>,
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(Rc<List>),
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

impl Atom {
    pub fn new(name: &str) -> Self {
        Self {
            name: Box::from(name),
        }
    }
}

impl List {
    pub fn empty() -> Rc<Self> {
        Rc::new(List::EmptyList {})
    }
}

impl From<&source::Atom> for Value {
    fn from(src: &source::Atom) -> Self {
        Value::Atom(Atom {
            name: src.name.clone(),
        })
    }
}

impl From<&source::List> for Value {
    fn from(src: &source::List) -> Self {
        let mut head = List::empty();
        for v in src.list.iter().rev() {
            head = Rc::new(List::Head {
                head: v.into(),
                tail: head,
            })
        }

        Value::List(head)
    }
}

impl From<&source::Value> for Value {
    fn from(src: &source::Value) -> Self {
        match src {
            source::Value::Atom(a) => a.into(),
            source::Value::List(l) => l.into(),
        }
    }
}
