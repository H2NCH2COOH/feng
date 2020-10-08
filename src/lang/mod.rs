use std::collections::HashMap;
use std::rc::Rc;
use std::vec::Vec;

mod chars;
mod parser;
pub mod error;

struct DebugInfo {
    filename: String,
    lineno: u64,
    charno: u64,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

#[derive(Debug)]
enum List {
    EmptyList,
    Head {
        head: Value,
        tail: Rc<List>,
    },
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(Rc<List>),
    //Lambda(Lambda),
}

pub fn parse<S>(stream: S) -> Result<Vec<Value>, error::Error>
where
    S: Iterator<Item = std::io::Result<u8>>,
{
    parser::parse(chars::Chars::new(stream))
}
