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
    level: u64,
    source_info: &SourceInfo,
) -> Result<(), Error> {
    fn define_recur(
        key: &Atom,
        val: &Value,
        ctx: &mut Context,
        level: u64,
        level_now: u64,
        source_info: &SourceInfo,
    ) -> Result<(), Error> {
        if level > 0 {
            match &mut ctx.parent {
                Some(parent) => {
                    define_recur(key, val, parent, level - 1, level_now + 1, source_info)
                },
                None => Err(Error::BadCtxLevel {
                    source_info: source_info.clone(),
                    level_required: level + level_now,
                    level_max: level_now,
                }),
            }
        } else {
            ctx.map.insert(key.clone(), val.clone());
            Ok(())
        }
    }

    define_recur(key, val, ctx, level, 0, source_info)
}

fn lookup(key: &Atom, ctx: &Context, level: u64, source_info: &SourceInfo) -> Result<Value, Error> {
    fn lookup_recur(
        key: &Atom,
        ctx: &Context,
        level: u64,
        level_now: u64,
        source_info: &SourceInfo,
    ) -> Result<Value, Error> {
        if level > 0 {
            match &ctx.parent {
                Some(parent) => lookup_recur(key, parent, level - 1, level_now + 1, source_info),
                None => Err(Error::BadCtxLevel {
                    source_info: source_info.clone(),
                    level_required: level + level_now,
                    level_max: level_now,
                }),
            }
        } else {
            match ctx.map.get(key) {
                Some(val) => Ok(val.clone()),
                None => match &ctx.parent {
                    Some(parent) => lookup_recur(key, parent, 0, level_now + 1, source_info),
                    None => Ok(Value::Atom(key.clone())),
                },
            }
        }
    }
    lookup_recur(key, ctx, level, 0, source_info)
}

pub fn eval_source(src: &[source::Value]) -> Result<Value, Error> {
    todo!()
}

fn eval(ctx: &Context, val: &Value, source_info: &SourceInfo) -> Result<Value, Error> {
    todo!()
}
