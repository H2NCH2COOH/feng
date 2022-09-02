use super::source;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: Rc<str>,
}

#[derive(Clone, Debug)]
pub enum List {
    Empty,
    Head(Rc<ListHead>),
}

#[derive(Debug)]
pub struct ListHead {
    pub val: Value,
    pub tail: List,
}

#[derive(Clone, Debug)]
pub enum ArgList {
    Vargs(Atom),
    Args(Rc<[Atom]>),
}

#[derive(Clone, Debug)]
pub struct Fexpr {
    pub arg_list: ArgList,
    pub body: List,
}

macro_rules! define_funcs {
    ( $( ( $name:expr, $enum_id:ident ) ),* ) => {
        #[derive(Clone, Copy, Debug)]
        pub enum Function {
            $( $enum_id, )*
        }

        pub const NAMED_FUNCS: &'static [(&'static str, Function)] = &[
            $( ($name, Function::$enum_id), )*
        ];

        impl std::fmt::Display for Function {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( Function::$enum_id => write!(f, $name), )*
                }
            }
        }
    }
}

define_funcs!(
    ("puts", Print),
    ("puts!", PrintF),
    ("cond", Cond),
    ("eval", Eval),
    ("eval!", EvalF),
    ("upeval", UpEval),
    ("upeval!", UpEvalF),
    ("define", Define),
    ("define!", DefineF),
    ("atom-concat", AtomConcat),
    ("atom-concat!", AtomConcatF),
    ("atom-eq", AtomEqual),
    ("atom-eq!", AtomEqualF),
    ("atom?", IsAtom),
    ("atom?!", IsAtomF),
    ("list?", IsList),
    ("list?!", IsListF),
    ("begin!", BeginF),
    ("quote!", QuoteF),
    ("list", List),
    ("car", Car),
    ("car!", CarF),
    ("cdr", Cdr),
    ("cdr!", CdrF),
    ("cons", Cons),
    ("cons!", ConsF)
);

#[derive(Clone, Debug)]
pub enum Value {
    SourceAtom(source::Atom),
    SourceList(source::List),
    Atom(Atom),
    List(List),
    Fexpr(Fexpr),
    Function(Function),
}

pub const EMPTY_LIST: Value = Value::List(List::Empty);
pub const FALSE: Value = EMPTY_LIST;
thread_local! {
    pub static TRUE: Value = Value::Atom(Atom::new("true"));
}

impl Atom {
    pub fn new(name: &str) -> Self {
        Atom {
            name: Rc::from(name),
        }
    }
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

impl From<Atom> for Value {
    fn from(that: Atom) -> Self {
        Value::Atom(that)
    }
}

impl List {}

impl From<&Value> for bool {
    fn from(that: &Value) -> Self {
        match that {
            Value::List(List::Empty) => false,
            _ => true,
        }
    }
}

impl From<Value> for bool {
    fn from(that: Value) -> Self {
        (&that).into()
    }
}

impl From<List> for Value {
    fn from(that: List) -> Self {
        Value::List(that)
    }
}

impl From<&source::Atom> for Atom {
    fn from(that: &source::Atom) -> Self {
        Self {
            name: that.name.clone(),
        }
    }
}

impl From<&source::Value> for Value {
    fn from(that: &source::Value) -> Self {
        match that {
            source::Value::Atom(a) => Value::SourceAtom(a.clone()),
            source::Value::List(l) => Value::SourceList(l.clone()),
        }
    }
}

impl From<&[source::Value]> for List {
    fn from(that: &[source::Value]) -> Self {
        let mut head = List::Empty;
        for v in that.iter().rev() {
            head = List::Head(Rc::new(ListHead {
                val: v.into(),
                tail: head,
            }));
        }

        head
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}'", self.name)
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            List::Empty => write!(f, "()"),
            List::Head(head) => match head.tail {
                List::Empty => write!(f, "({})", head.val),
                List::Head(_) => write!(f, "({} ...)", head.val),
            },
        }
    }
}

impl std::fmt::Display for Fexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.arg_list {
            ArgList::Args(list) => {
                write!(f, "(fexpr! (")?;
                if !list.is_empty() {
                    write!(f, "{}", list[0])?;
                    for atom in &list[1..] {
                        write!(f, " {}", atom)?;
                    }
                }
                write!(f, ") {})", self.body)
            }
            ArgList::Vargs(atom) => write!(f, "(fexpr! {} {})", atom, self.body),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atom(atom) => write!(f, "{}", atom),
            Self::List(list) => write!(f, "{}", list),
            Self::SourceAtom(source_atom) => write!(f, "{}", source_atom),
            Self::SourceList(source_list) => write!(f, "{}", source_list),
            Self::Fexpr(fexpr) => write!(f, "{}", fexpr),
            Self::Function(func) => write!(f, "{}", func),
        }
    }
}
