use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::rc::Rc;
use std::vec::Vec;

mod chars;
mod error;
mod parser;
mod printer;

pub use error::Error;

#[derive(Clone, Debug)]
pub struct SourceInfo {
    name: Rc<String>,
    lineno: usize,
    charno: usize,
}

#[derive(Clone, Debug)]
pub struct Atom {
    name: String,
    source_info: Option<SourceInfo>,
}

#[derive(Debug)]
struct Scope {
    parent: Option<Rc<Scope>>,
    table: HashMap<Atom, Value>,
}

#[derive(Clone, Debug)]
enum LambdaArgs {
    Vargs(Atom),
    Args(Vec<Atom>),
}

#[derive(Clone, Debug)]
struct Lambda {
    scope: Rc<Scope>,
    args: LambdaArgs,
    body: Rc<List>,
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
    Lambda(Lambda),
    Function(String), // TODO
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

pub fn parse<S>(name: &str, stream: &mut S) -> Result<Vec<Value>, Error>
where
    S: Iterator<Item = std::io::Result<u8>>,
{
    parser::parse(name, chars::Chars::new(stream))
}

pub fn print<W: Write>(out: &mut W, val: &Value) -> Result<(), Error> {
    printer::print(out, val)
}
