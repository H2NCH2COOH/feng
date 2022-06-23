use std::boxed::Box;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SourceInfo {
    pub name: Rc<str>,
    pub lineno: usize,
    pub charno: usize,
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: Box<str>,
    pub source_info: Option<SourceInfo>,
}

#[derive(Debug)]
pub enum List {
    EmptyList {
        source_info: Option<SourceInfo>,
    },
    Head {
        head: Value,
        tail: Rc<List>,
        source_info: Option<SourceInfo>,
    },
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(Rc<List>),
}
