use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SourceInfo {
    pub name: Rc<str>,
    pub lineno: usize,
    pub charno: usize,
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: Box<str>, // Might use Rc?
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug)]
pub struct List {
    pub list: Box<[Value]>, // Might use Rc?
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
