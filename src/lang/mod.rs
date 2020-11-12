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

#[derive(Debug)]
struct ClosureScope {
    table: HashMap<Atom, Value>,
}

#[derive(Clone, Debug)]
enum LambdaArgs {
    Vargs(Atom),
    Args(Vec<Atom>),
}

#[derive(Clone, Debug)]
struct Lambda {
    closure_scope: Rc<ClosureScope>,
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
pub enum Function {
    //TODO
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(Atom),
    List(Rc<List>),
    Lambda(Lambda),
    Function(Function),
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
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            source_info: None,
        }
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

fn lookup(atom: &Atom, scope: &Scope) -> Option<Value> {
    match scope.table.get(atom) {
        Some(v) => Some(v.clone()),
        None => match &scope.parent {
            Some(p) => lookup(atom, &p),
            None => None,
        },
    }
}

fn eval_args(list: &Rc<List>, scope: &Rc<Scope>) -> Result<List, Error> {
    todo!()
}

fn eval(val: &Value, scope: &Rc<Scope>) -> Result<Value, Error> {
    let mut curr_scope = Rc::new(Scope {
        parent: Some(scope.clone()),
        table: HashMap::new(),
    });

    loop {
        match val {
            // Lookup Atom and return itself when not found
            Value::Atom(atom) => {
                return Ok(lookup(atom, &curr_scope).unwrap_or(Value::Atom(atom.clone())))
            }
            Value::List(list) => match list.as_ref() {
                List::EmptyList { source_info: _ } => {
                    return Ok(Value::List(Rc::new(List::EmptyList { source_info: None })))
                }
                List::Head {
                    head,
                    tail,
                    source_info,
                } => {
                    let val = eval(head, scope)?;
                    match val {
                        Value::Function(_) => {
                            let args = eval_args(tail, &curr_scope)?;
                            todo!()
                        }
                        Value::Lambda(Lambda {
                            closure_scope,
                            args,
                            body,
                        }) => {
                            let args = eval_args(tail, &curr_scope)?;
                            todo!()
                        }
                        _ => {
                            return Err(Error::ValueErr {
                                source_info: source_info.clone().unwrap(),
                                msg: format!("Can't call value: {:?}", val),
                            })
                        }
                    }
                }
            },
            _ => panic!("Can't evaluate {:?}", val),
        }
    }
}
