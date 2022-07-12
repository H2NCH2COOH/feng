use super::error::Error;
use super::source;
use super::source::SourceInfo;
use super::value::{ArgList, Atom, Fexpr, Function, List, ListHead, Value};
use std::collections::HashMap;
use std::rc::Rc;

struct Context<'a> {
    parent: Option<&'a Self>,
    map: HashMap<Atom, Value>,
}

enum ListRef<'a> {
    Value(&'a List),
    Source(&'a source::List),
}

impl<'a> From<&'a List> for ListRef<'a> {
    fn from(that: &'a List) -> Self {
        Self::Value(that)
    }
}

impl<'a> From<&'a source::List> for ListRef<'a> {
    fn from(that: &'a source::List) -> Self {
        Self::Source(that)
    }
}

impl<'a> From<&ListRef<'a>> for Value {
    fn from(that: &ListRef<'a>) -> Self {
        match *that {
            ListRef::Value(v) => Value::List(v.clone()),
            ListRef::Source(s) => Value::SourceList(s.clone()),
        }
    }
}

pub struct ListIter<'a> {
    next: &'a List,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            List::Empty => None,
            List::Head(head) => {
                self.next = &head.tail;
                Some(head.val.clone())
            }
        }
    }
}

impl<'a> IntoIterator for &'a List {
    type Item = Value;
    type IntoIter = ListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ListIter { next: self }
    }
}

pub struct SourceListIter<'a> {
    base: &'a source::List,
    idx: usize,
}

impl<'a> Iterator for SourceListIter<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.base.list.len() {
            let ret = (&self.base.list[self.idx]).into();
            self.idx += 1;
            Some(ret)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a source::List {
    type Item = Value;
    type IntoIter = SourceListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SourceListIter { base: self, idx: 0 }
    }
}

fn car(list: &ListRef) -> Option<Value> {
    match *list {
        ListRef::Value(v) => match v {
            List::Empty => None,
            List::Head(head) => Some(head.val.clone()),
        },
        ListRef::Source(s) => {
            if s.list.is_empty() {
                None
            } else {
                Some((&s.list[0]).into())
            }
        }
    }
}

fn cdr(list: &ListRef) -> Option<List> {
    match *list {
        ListRef::Value(v) => match v {
            List::Empty => None,
            List::Head(head) => Some(head.tail.clone()),
        },
        ListRef::Source(s) => match s.list.len() {
            0 => None,
            1 => Some(List::Empty),
            _ => Some((&s.list[1..]).into()),
        },
    }
}

fn cons(val: &Value, list: &ListRef) -> Value {
    Value::List(List::Head(Rc::new(match *list {
        ListRef::Value(v) => ListHead {
            val: val.clone(),
            tail: v.clone(),
        },
        ListRef::Source(s) => ListHead {
            val: val.clone(),
            tail: (&s.list[..]).into(),
        },
    })))
}

fn list_len(list: &ListRef) -> usize {
    match list {
        ListRef::Value(v) => match v {
            List::Empty => 0,
            List::Head(head) => list_len(&ListRef::Value(&head.tail)) + 1,
        },
        ListRef::Source(s) => s.list.len(),
    }
}

fn define(
    key: &Atom,
    val: Value,
    ctx: &mut Context,
    _source_info: &SourceInfo,
) -> Result<(), Error> {
    ctx.map.insert(key.clone(), val);
    Ok(())
}

fn lookup(key: &Atom, ctx: &Context, source_info: &SourceInfo) -> Result<Value, Error> {
    match ctx.map.get(key) {
        Some(val) => Ok(val.clone()),
        None => match &ctx.parent {
            Some(parent) => lookup(key, parent, source_info),
            None => Ok(Value::Atom(key.clone())),
        },
    }
}

pub fn eval_source(src: &[source::Value]) -> Result<Value, Error> {
    todo!()
}

fn eval(val: &Value, ctx: &mut Context, source_info: &SourceInfo) -> Result<Value, Error> {
    match val {
        Value::Atom(atom) => lookup(atom, ctx, source_info),
        Value::List(list) => call(list.into(), ctx, source_info),
        Value::SourceList(list) => call(list.into(), ctx, &list.source_info),
        _ => Err(Error::CantEval {
            source_info: source_info.clone(),
            val: val.clone(),
        }),
    }
}

fn upeval(val: &Value, ctx: &mut Context, source_info: &SourceInfo) -> Result<Value, Error> {
    match ctx.parent {
        Some(parent) => todo!(),
        None => Err(Error::NoUpCtx {
            source_info: source_info.clone(),
        }),
    }
}

fn call(list: ListRef, ctx: &mut Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let callable = car(&list).ok_or(Error::CantEval {
        source_info: source_info.clone(),
        val: (&list).into(),
    })?;

    let callable = match &callable {
        Value::Atom(atom) => lookup(atom, ctx, source_info)?,
        Value::SourceAtom(atom) => lookup(&atom.into(), ctx, source_info)?,
        _ => callable,
    };

    let args = cdr(&list).unwrap_or(List::Empty);

    match &callable {
        Value::Fexpr(fexpr) => call_fexpr(fexpr, &args, ctx, source_info),
        Value::Function(func) => call_function(func, &args, ctx, source_info),
        _ => Err(Error::CantCall {
            source_info: source_info.clone(),
            val: callable.clone(),
        }),
    }
}

fn apply_args(
    arg_list: &ArgList,
    args: &List,
    ctx: &mut Context,
    source_info: &SourceInfo,
) -> Result<(), Error> {
    match arg_list {
        ArgList::Vargs(atom) => define(atom, Value::List(args.clone()), ctx, source_info),
        ArgList::Args(list) => {
            let expected = list.len();
            let found = list_len(&ListRef::Value(args));
            if expected != found {
                Err(Error::BadArgsNum {
                    source_info: source_info.clone(),
                    expected,
                    found,
                })
            } else {
                for (atom, val) in std::iter::zip(list.iter(), args.into_iter()) {
                    define(atom, val, ctx, source_info)?;
                }
                Ok(())
            }
        }
    }
}

fn call_fexpr(
    fexpr: &Fexpr,
    args: &List,
    parent_ctx: &mut Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    let mut ctx = Context {
        parent: Some(parent_ctx),
        map: HashMap::new(),
    };
    apply_args(&fexpr.arg_list, args, &mut ctx, source_info)?;
    eval(&Value::List(fexpr.body.clone()), &mut ctx, source_info)
}

fn call_function(
    func: &Function,
    args: &List,
    parent_ctx: &mut Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    todo!()
}
