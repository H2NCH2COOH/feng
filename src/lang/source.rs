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
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug)]
pub struct List {
    pub list: Box<[Value]>,
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(List),
}
