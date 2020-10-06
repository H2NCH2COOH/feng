use std::collections::HashMap;
use std::rc::Rc;
use std::vec::Vec;

mod chars;

pub mod error;

struct DebugInfo {
    filename: String,
    lineno: u64,
    charno: u64,
}

#[derive(Clone)]
struct Atom {
    name: String,
}

struct Scope {
    parent: Option<Rc<Scope>>,
    tab: HashMap<String, Value>,
}

#[derive(Clone)]
enum Arguments {
    Vargs(Atom),
    Args(Vec<Atom>),
}

#[derive(Clone)]
struct Lambda {
    scope: Rc<Scope>,
    args: Arguments,
    body: Rc<List>,
}

struct List {
    head: Option<Value>,
    tail: Option<Rc<List>>,
}

#[derive(Clone)]
enum Value {
    Atom(Atom),
    List(Rc<List>),
    Lambda(Lambda),
}
