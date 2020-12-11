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
struct SourceInfo {
    name: Rc<String>,
    lineno: usize,
    charno: usize,
}

#[derive(Clone, Debug)]
struct Atom {
    name: String,
    source_info: Option<SourceInfo>,
}

#[derive(Debug)]
struct TailRecursion {
    source_info: Option<SourceInfo>,
    args: Rc<List>,
}

#[derive(Debug)]
struct Scope {
    parent: Option<Rc<Scope>>,
    table: HashMap<Atom, Value>,
    recursion_point: Option<Lambda>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LambdaArgs {
    Vargs(Atom),
    Args(Vec<Atom>),
}

#[derive(Clone, Debug)]
struct Lambda {
    args: LambdaArgs,
    body: Rc<List>,
}

#[derive(Debug)]
enum List {
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
enum Function {
    Begin,
    Cond,
    Quote,
    List,
    Len,
    //TODO
}

#[derive(Clone, Debug)]
enum Value {
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

impl PartialEq for Lambda {
    fn eq(&self, other: &Self) -> bool {
        self.args == other.args && Rc::ptr_eq(&self.body, &other.body)
    }
}

impl Eq for Lambda {}

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

impl List {
    fn empty() -> Rc<Self> {
        Rc::new(List::EmptyList { source_info: None })
    }
}

fn cons(head: &Value, tail: &Rc<List>) -> Rc<List> {
    Rc::new(List::Head {
        head: head.clone(),
        tail: tail.clone(),
        source_info: None,
    })
}

fn len(list: &Rc<List>) -> usize {
    let mut list = list.as_ref();
    let mut len = 0usize;

    loop {
        match list {
            List::EmptyList { source_info: _ } => break,
            List::Head {
                head: _,
                tail,
                source_info: _,
            } => {
                list = tail;
                len += 1;
            }
        }
    }

    len
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

fn def(atom: &Atom, value: &Value, scope: &mut Rc<Scope>) {
    Rc::get_mut(scope)
        .unwrap()
        .table
        .insert(atom.clone(), value.clone());
}

fn eval_value(val: &Value, scope: &Rc<Scope>) -> Result<Value, Error> {
    match val {
        // Lookup Atom and return itself when not found
        Value::Atom(atom) => Ok(lookup(atom, &scope).unwrap_or(Value::Atom(atom.clone()))),
        Value::List(list) => match list.as_ref() {
            // Empty list as itself
            List::EmptyList { source_info: _ } => Ok(Value::List(list.clone())),
            List::Head {
                head,
                tail,
                source_info,
            } => match eval_value(head, scope)? {
                Value::Function(_) => todo!(),
                Value::Lambda(ref lambda) => {
                    eval_lambda(lambda, tail, scope, source_info.as_ref().unwrap())
                }
                _ => Err(Error::ValueErr {
                    source_info: source_info.clone().unwrap(),
                    msg: format!("Can't call value: {:?}", val),
                }),
            },
        },
        _ => panic!("Can't evaluate {:?}", val),
    }
}

fn eval_args(list: &Rc<List>, scope: &Rc<Scope>) -> Result<Rc<List>, Error> {
    match list.as_ref() {
        List::EmptyList { source_info: _ } => Ok(List::empty()),
        List::Head {
            head,
            tail,
            source_info,
        } => {
            let v = eval_value(head, scope)?;
            Ok(cons(&v, &eval_args(tail, scope)?))
        }
    }
}

fn eval_lambda(
    lambda: &Lambda,
    args: &Rc<List>,
    scope: &Rc<Scope>,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    let mut curr_scope = Rc::new(Scope {
        parent: Some(scope.clone()),
        table: HashMap::new(),
        recursion_point: Some(lambda.clone()),
    });

    loop {
        match &lambda.args {
            LambdaArgs::Vargs(name) => {
                def(name, &Value::List(args.clone()), &mut curr_scope);
            }
            LambdaArgs::Args(names) => {
                if names.len() != len(args) {
                    return Err(Error::ArgumentNumberErr {
                        source_info: source_info.clone(),
                        expected: names.len(),
                        actual: len(args),
                    });
                }

                let mut i = 0usize;
                let mut p = args.as_ref();
                loop {
                    match p {
                        List::EmptyList { source_info: _ } => break,
                        List::Head {
                            head,
                            tail,
                            source_info: _,
                        } => {
                            def(&names[i], head, &mut curr_scope);
                            p = tail;
                            i += 1;
                        }
                    }
                }
            }
        }

        todo!()
    }
}
