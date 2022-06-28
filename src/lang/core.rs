use super::error::Error;
use super::source;
use super::source::SourceInfo;
use super::value::{Atom, Fexpr, List, Value};
use std::collections::HashMap;

struct Context<'a> {
    parent: Option<&'a mut Self>,
    map: HashMap<Atom, Value>,
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
        Value::List(list) => todo!(),
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
