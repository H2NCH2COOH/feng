use super::error::Error;
use super::source;
use super::source::SourceInfo;
use super::value::{Atom, Fexpr, List, ListHead, Value};
use std::collections::HashMap;
use std::rc::Rc;

struct Context<'a> {
    parent: Option<&'a mut Self>,
    map: HashMap<Atom, Value>,
}

#[derive(Debug)]
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

impl<'a> From<ListRef<'a>> for Value {
    fn from(that: ListRef<'a>) -> Self {
        match that {
            ListRef::Value(v) => Value::List(v.clone()),
            ListRef::Source(s) => Value::SourceList(s.clone()),
        }
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

fn cdr(list: &ListRef) -> Option<Value> {
    match *list {
        ListRef::Value(v) => match v {
            List::Empty => None,
            List::Head(head) => match head.tail {
                List::Empty => None,
                List::Head(_) => Some(Value::List(List::Head(head.clone()))),
            },
        },
        ListRef::Source(s) => match s.list.len() {
            0..=1 => None,
            _ => Some(Value::List((&s.list[1..]).into())),
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

fn define(
    key: &Atom,
    val: &Value,
    ctx: &mut Context,
    source_info: &SourceInfo,
) -> Result<(), Error> {
    ctx.map.insert(key.clone(), val.clone());
    Ok(())
}

fn updefine(
    key: &Atom,
    val: &Value,
    ctx: &mut Context,
    source_info: &SourceInfo,
) -> Result<(), Error> {
    match &mut ctx.parent {
        Some(parent) => define(key, val, parent, source_info),
        None => Err(Error::NoUpCtx {
            source_info: source_info.clone(),
        }),
    }
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

fn uplookup(key: &Atom, ctx: &Context, source_info: &SourceInfo) -> Result<Value, Error> {
    match &ctx.parent {
        Some(parent) => lookup(key, parent, source_info),
        None => Err(Error::NoUpCtx {
            source_info: source_info.clone(),
        }),
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
    match &mut ctx.parent {
        Some(parent) => eval(val, parent, source_info),
        None => Err(Error::NoUpCtx {
            source_info: source_info.clone(),
        }),
    }
}

fn call(list: ListRef, ctx: &mut Context, source_info: &SourceInfo) -> Result<Value, Error> {
    return Err(Error::CantEval {
        source_info: source_info.clone(),
        val: list.into(),
    });

    todo!()
}
