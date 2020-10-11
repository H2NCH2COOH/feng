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
    filename: Option<String>,
    lineno: u64,
    charno: u64,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Atom {
    name: String,
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
    EmptyList,
    Head { head: Value, tail: Rc<List> },
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(Rc<List>),
    Lambda(Lambda),
    Function(String), // TODO
}

pub fn parse<S>(name: Option<&str>, stream: &mut S) -> Result<Vec<Value>, Error>
where
    S: Iterator<Item = std::io::Result<u8>>,
{
    parser::parse(name, chars::Chars::new(stream))
}

pub fn print<W: Write>(out: &mut W, val: &Value) -> Result<(), Error> {
    printer::print(out, val)
}
