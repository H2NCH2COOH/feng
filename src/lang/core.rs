use super::error::Error;
use super::source;
use super::source::SourceInfo;
use super::value::{
    ArgList, Atom, Fexpr, Function, List, ListHead, Value, EMPTY_LIST, FALSE, TRUE,
};
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

struct Context<'a> {
    parent: Option<&'a Self>,
    map: HashMap<Atom, Value>,
}

#[derive(Copy, Clone)]
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

pub struct ListIter<'a> {
    next: &'a List,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            List::Empty => None,
            List::Head(head) => {
                self.next = &head.tail;
                Some(&head.val)
            }
        }
    }
}

impl<'a> IntoIterator for &'a List {
    type Item = &'a Value;
    type IntoIter = ListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ListIter { next: self }
    }
}

fn car(list: ListRef) -> Option<Value> {
    match list {
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

fn cdr(list: ListRef) -> Option<List> {
    match list {
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

fn cons(val: &Value, list: ListRef) -> List {
    List::Head(Rc::new(match list {
        ListRef::Value(v) => ListHead {
            val: val.clone(),
            tail: v.clone(),
        },
        ListRef::Source(s) => ListHead {
            val: val.clone(),
            tail: (&s.list[..]).into(),
        },
    }))
}

fn list_len(list: ListRef) -> usize {
    match list {
        ListRef::Value(v) => match v {
            List::Empty => 0,
            List::Head(head) => list_len(ListRef::Value(&head.tail)) + 1,
        },
        ListRef::Source(s) => s.list.len(),
    }
}

fn define(key: &Atom, val: Value, ctx: &mut Context) {
    ctx.map.insert(key.clone(), val);
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

fn create_root_context() -> Context<'static> {
    let mut ctx = Context {
        parent: None,
        map: HashMap::new(),
    };

    for (name, func) in super::value::NAMED_FUNCS {
        define(&Atom::new(name), Value::Function(*func), &mut ctx);
    }

    ctx
}

pub fn eval_source(src: &[source::Value]) -> Result<Value, Error> {
    let mut ctx = create_root_context();

    src.iter().try_fold(EMPTY_LIST, |_, val| {
        eval(&val.into(), &mut ctx, val.source_info())
    })
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

fn call(list: ListRef, ctx: &mut Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let callable = car(list).ok_or(Error::CantEval {
        source_info: source_info.clone(),
        val: list.into(),
    })?;

    let callable = match &callable {
        Value::Atom(atom) => lookup(atom, ctx, source_info)?,
        Value::SourceAtom(atom) => lookup(&atom.into(), ctx, source_info)?,
        _ => callable,
    };

    let args = cdr(list).unwrap_or(List::Empty);

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
        ArgList::Vargs(atom) => {
            define(atom, Value::List(args.clone()), ctx);
            Ok(())
        }
        ArgList::Args(list) => {
            let expected = list.len();
            let found = list_len(ListRef::Value(args));
            if expected != found {
                Err(Error::BadArgsNum {
                    source_info: source_info.clone(),
                    expected,
                    found,
                })
            } else {
                for (atom, val) in std::iter::zip(list.iter(), args.into_iter()) {
                    define(atom, val.clone(), ctx);
                }
                Ok(())
            }
        }
    }
}

fn eval_args(args: &List, ctx: &mut Context, source_info: &SourceInfo) -> Result<List, Error> {
    Ok(match args {
        List::Empty => List::Empty,
        List::Head(head) => cons(
            &eval(&head.val, ctx, source_info)?,
            (&eval_args(&head.tail, ctx, source_info)?).into(),
        ),
    })
}

fn call_fexpr(
    fexpr: &Fexpr,
    args: &List,
    parent_ctx: &Context,
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
    parent_ctx: &Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    todo!()
}

fn func_puts(
    args: &List,
    _parent_ctx: &Context,
    _source_info: &SourceInfo,
) -> Result<Value, Error> {
    for v in args {
        super::print(&mut std::io::stdout(), &v)?;
    }
    std::io::stdout().write_all(b"\n")?;

    Ok(EMPTY_LIST)
}

fn func_cond(args: &List, parent_ctx: &Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let mut ctx = Context {
        parent: Some(parent_ctx),
        map: HashMap::new(),
    };

    let mut iter = args.into_iter();
    loop {
        let test = match iter.next() {
            None => break Ok(EMPTY_LIST),
            Some(v) => v,
        };
        let val = match iter.next() {
            None => {
                break Err(Error::BadFuncArgs {
                    source_info: source_info.clone(),
                    msg: "cond: must have an even number of arguments".to_string(),
                })
            }
            Some(v) => v,
        };

        let test = eval(&test, &mut ctx, source_info)?;
        if test.into() {
            break eval(&val, &mut ctx, source_info);
        }
    }
}

fn func_eval(args: &List, parent_ctx: &Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let val = match args {
        List::Empty => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "eval: must have an argument".to_string(),
        }),
        List::Head(head) => match head.tail {
            List::Empty => Ok(&head.val),
            List::Head(_) => Err(Error::BadFuncArgs {
                source_info: source_info.clone(),
                msg: "eval: must have only one argument".to_string(),
            }),
        },
    }?;

    let mut ctx = Context {
        parent: Some(parent_ctx),
        map: HashMap::new(),
    };

    eval(val, &mut ctx, source_info)
}

fn func_upeval(
    args: &List,
    parent_ctx: &Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    let val = match args {
        List::Empty => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "upeval: must have an argument".to_string(),
        }),
        List::Head(head) => match head.tail {
            List::Empty => Ok(&head.val),
            List::Head(_) => Err(Error::BadFuncArgs {
                source_info: source_info.clone(),
                msg: "upeval: must have only one argument".to_string(),
            }),
        },
    }?;

    let parent_ctx = match parent_ctx.parent {
        Some(p) => Ok(p),
        None => Err(Error::NoUpCtx {
            source_info: source_info.clone(),
        }),
    }?;

    let mut ctx = Context {
        parent: Some(parent_ctx),
        map: HashMap::new(),
    };

    eval(val, &mut ctx, source_info)
}

fn func_define(args: &List, ctx: &mut Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let mut args_iter = args.into_iter();

    let key = match args_iter.next() {
        Some(Value::Atom(a)) => Ok(a),
        Some(_) => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "define: the first argument must be an atom".to_string(),
        }),
        None => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "define: must have two arguments".to_string(),
        }),
    }?;

    let val = match args_iter.next() {
        Some(v) => Ok(v),
        None => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "define: must have two arguments".to_string(),
        }),
    }?;

    if let Some(_) = args_iter.next() {
        return Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "define: must have only two arguments".to_string(),
        });
    }

    define(&key, val.clone(), ctx);

    Ok(EMPTY_LIST)
}

fn func_atom_concat(
    args: &List,
    _parent_ctx: &Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    let mut idx = 0;
    let mut buf = String::with_capacity(1024);

    for v in args.into_iter() {
        idx += 1;
        let atom = match v {
            Value::Atom(a) => Ok(&a.name),
            _ => Err(Error::BadFuncArgs {
                source_info: source_info.clone(),
                msg: format!("atom-concat: argument #{} is not an atom", idx),
            }),
        }?;
        buf.push_str(atom);
    }

    if idx == 0 {
        return Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "atom-concat: must have at least one argument".to_string(),
        });
    }

    Ok(Atom::new(&buf).into())
}

fn func_atom_eq(
    args: &List,
    _parent_ctx: &Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    let mut args_iter = args.into_iter();

    let atom = match args_iter.next() {
        Some(Value::Atom(a)) => Ok(a),
        Some(_) => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "atom-eq?: argument #1 is not an atom".to_string(),
        }),
        _ => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "atom-eq?: must have at least one argument".to_string(),
        }),
    }?;

    let mut idx = 1;
    for v in args_iter {
        idx += 1;
        if let Value::Atom(a) = v {
            if a != atom {
                return Ok(FALSE);
            }
        } else {
            return Err(Error::BadFuncArgs {
                source_info: source_info.clone(),
                msg: format!("atom-eq?: argument #{} is not an atom", idx),
            });
        }
    }

    Ok(TRUE())
}

fn func_is_atom(
    args: &List,
    _parent_ctx: &Context,
    _source_info: &SourceInfo,
) -> Result<Value, Error> {
    for v in args.into_iter() {
        match v {
            Value::Atom(_) => (),
            _ => return Ok(FALSE),
        }
    }

    Ok(TRUE())
}

fn func_is_list(
    args: &List,
    _parent_ctx: &Context,
    _source_info: &SourceInfo,
) -> Result<Value, Error> {
    for v in args.into_iter() {
        match v {
            Value::List(_) => (),
            _ => return Ok(FALSE),
        }
    }

    Ok(TRUE())
}

fn func_begin(args: &List, parent_ctx: &Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let mut ctx = Context {
        parent: Some(parent_ctx),
        map: HashMap::new(),
    };

    let mut rst = EMPTY_LIST;
    for v in args {
        rst = eval(v, &mut ctx, source_info)?;
    }
    Ok(rst)
}
