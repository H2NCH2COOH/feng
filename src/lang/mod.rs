use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use std::vec::Vec;

mod chars;
mod error;
mod parser;
mod printer;

pub use error::Error;

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
    Head { head: Value, tail: Rc<List> },
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(Rc<List>),
    //Lambda(Lambda),
}

pub fn parse<S>(stream: &mut S) -> Result<Vec<Value>, Error>
where
    S: Iterator<Item = std::io::Result<u8>>,
{
    parser::parse(chars::Chars::new(stream))
}

pub fn print<W: Write>(out: &mut W, val: &Value) -> Result<(), Error> {
    printer::print(out, val)
}
