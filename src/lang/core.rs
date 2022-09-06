use super::error::Error;
use super::source;
use super::source::SourceInfo;
use super::value::{
    ArgList, Atom, Fexpr, Function, List, ListHead, Value, EMPTY_LIST, FALSE, TRUE,
};
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

#[derive(Debug)]
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
            List::Head(head) => list_len((&head.tail).into()) + 1,
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
            None => Ok(key.clone().into()),
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
        Value::SourceAtom(atom) => lookup(&atom.into(), ctx, source_info),
        Value::SourceList(list) => call(list.into(), ctx, &list.source_info),
        _ => Err(Error::CantEval {
            source_info: source_info.clone(),
            val: val.clone(),
        }),
    }
}

fn call(list: ListRef, ctx: &mut Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let callable = match car(list) {
        Some(v) => v,
        None => return Ok(EMPTY_LIST),
    };

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
            define(atom, args.clone().into(), ctx);
            Ok(())
        }
        ArgList::Args(list) => {
            let expected = list.len();
            let found = list_len(args.into());
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
    eval(&fexpr.body.clone().into(), &mut ctx, source_info)
}

fn call_function(
    func: &Function,
    args: &List,
    ctx: &mut Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    match func {
        Function::Puts => func_puts(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::PutsF => func_puts(args, ctx, source_info),
        Function::Cond => func_cond(args, ctx, source_info),
        Function::Eval => func_eval(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::EvalF => func_eval(args, ctx, source_info),
        Function::UpEval => func_upeval(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::UpEvalF => func_upeval(args, ctx, source_info),
        Function::Define => func_define(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::DefineF => func_define(args, ctx, source_info),
        Function::AtomConcat => {
            func_atom_concat(&eval_args(args, ctx, source_info)?, ctx, source_info)
        }
        Function::AtomConcatF => func_atom_concat(args, ctx, source_info),
        Function::AtomEq => func_atom_eq(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::AtomEqF => func_atom_eq(args, ctx, source_info),
        Function::IsAtom => func_is_atom(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::IsAtomF => func_is_atom(args, ctx, source_info),
        Function::IsList => func_is_list(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::IsListF => func_is_list(args, ctx, source_info),
        Function::IsFexpr => func_is_fexpr(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::IsFexprF => func_is_fexpr(args, ctx, source_info),
        Function::BeginF => func_begin(args, ctx, source_info),
        Function::QuoteF => func_quote(args, ctx, source_info),
        Function::List => func_list(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::Fexpr => func_fexpr(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::FexprF => func_fexpr(args, ctx, source_info),
        Function::Car => func_car(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::CarF => func_car(args, ctx, source_info),
        Function::Cdr => func_cdr(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::CdrF => func_cdr(args, ctx, source_info),
        Function::Cons => func_cons(&eval_args(args, ctx, source_info)?, ctx, source_info),
        Function::ConsF => func_cons(args, ctx, source_info),
    }
}

fn func_puts(
    args: &List,
    _parent_ctx: &Context,
    _source_info: &SourceInfo,
) -> Result<Value, Error> {
    for v in args {
        super::print(&mut std::io::stdout(), v)?;
    }
    std::io::stdout().write_all(b"\n")?;

    Ok(EMPTY_LIST)
}

fn func_cond(args: &List, ctx: &mut Context, source_info: &SourceInfo) -> Result<Value, Error> {
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

        let test = eval(test, ctx, source_info)?;
        if test.into() {
            break eval(val, ctx, source_info);
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
        Some(Value::Atom(a)) => Ok(a.clone()),
        Some(Value::SourceAtom(a)) => Ok(a.into()),
        Some(_) => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "define: the first argument is not an atom".to_string(),
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
            msg: "define: must a second argument".to_string(),
        }),
    }?;

    if args_iter.next().is_some() {
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
        let name = match v {
            Value::Atom(a) => Ok(&a.name),
            Value::SourceAtom(a) => Ok(&a.name),
            _ => Err(Error::BadFuncArgs {
                source_info: source_info.clone(),
                msg: format!("atom-concat: argument #{} is not an atom", idx),
            }),
        }?;
        buf.push_str(name);
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

    let name = match args_iter.next() {
        Some(Value::Atom(a)) => Ok(&a.name),
        Some(Value::SourceAtom(a)) => Ok(&a.name),
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
        match v {
            Value::Atom(a) => {
                if &a.name != name {
                    return Ok(FALSE);
                }
            }
            Value::SourceAtom(a) => {
                if &a.name != name {
                    return Ok(FALSE);
                }
            }
            _ => {
                return Err(Error::BadFuncArgs {
                    source_info: source_info.clone(),
                    msg: format!("atom-eq?: argument #{} is not an atom", idx),
                })
            }
        };
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
            Value::SourceAtom(_) => (),
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

fn func_is_fexpr(
    args: &List,
    _parent_ctx: &Context,
    _source_info: &SourceInfo,
) -> Result<Value, Error> {
    for v in args.into_iter() {
        match v {
            Value::Fexpr(_) => (),
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

fn func_quote(
    args: &List,
    _parent_ctx: &Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    let mut args_iter = args.into_iter();

    let val = args_iter.next().ok_or(Error::BadFuncArgs {
        source_info: source_info.clone(),
        msg: "quote: must have an argument".to_string(),
    })?;

    if args_iter.next().is_some() {
        return Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "quote: must have only one argument".to_string(),
        });
    }

    Ok(val.clone())
}

fn func_list(
    args: &List,
    _parent_ctx: &Context,
    _source_info: &SourceInfo,
) -> Result<Value, Error> {
    Ok(args.clone().into())
}

fn func_fexpr(
    args: &List,
    _parent_ctx: &Context,
    source_info: &SourceInfo,
) -> Result<Value, Error> {
    let mut args_iter = args.into_iter();

    let arg_list = match args_iter.next() {
        Some(Value::Atom(a)) => Ok(ArgList::Vargs(a.clone())),
        Some(Value::SourceAtom(a)) => Ok(ArgList::Vargs(a.into())),
        Some(Value::List(l)) => {
            let mut atom_vec = Vec::new();
            for v in l.into_iter() {
                match v {
                    Value::Atom(a) => {
                        atom_vec.push(a.clone());
                        Ok(())
                    }
                    Value::SourceAtom(a) => {
                        atom_vec.push(a.into());
                        Ok(())
                    }
                    _ => Err(Error::BadFuncArgs {
                        source_info: source_info.clone(),
                        msg: "fexpr: the argument list contains non-atom".to_string(),
                    }),
                }?;
            }
            Ok(ArgList::Args(Rc::from(atom_vec)))
        }
        Some(Value::SourceList(l)) => {
            let mut atom_vec = Vec::new();
            for v in l.list.iter() {
                match v {
                    source::Value::Atom(a) => {
                        atom_vec.push(a.into());
                        Ok(())
                    }
                    _ => Err(Error::BadFuncArgs {
                        source_info: source_info.clone(),
                        msg: "fexpr: the argument list contains non-atom".to_string(),
                    }),
                }?;
            }
            Ok(ArgList::Args(Rc::from(atom_vec)))
        }
        Some(_) => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "fexpr: the first argument must be either an atom or a list of atoms".to_string(),
        }),
        None => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "fexpr: must have two arguments".to_string(),
        }),
    }?;

    let body = match args_iter.next() {
        Some(Value::List(l)) => Ok(l.clone()),
        Some(Value::SourceList(l)) => Ok(l.into()),
        Some(_) => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "fexpr: the body is not a list".to_string(),
        }),
        _ => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "fexpr: must have a second argument".to_string(),
        }),
    }?;

    if args_iter.next().is_some() {
        return Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "fexpr: must have only two arguments".to_string(),
        });
    }

    Ok(Value::Fexpr(Fexpr { arg_list, body }))
}

fn func_car(args: &List, _parent_ctx: &Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let mut args_iter = args.into_iter();

    let list = match args_iter.next() {
        Some(Value::List(l)) => Ok(l),
        Some(_) => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "car: argument is not a list".to_string(),
        }),
        None => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "car: must have an argument".to_string(),
        }),
    }?;

    if args_iter.next().is_some() {
        return Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "car: must have only one argument".to_string(),
        });
    }

    Ok(car(list.into()).unwrap_or(EMPTY_LIST))
}

fn func_cdr(args: &List, _parent_ctx: &Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let mut args_iter = args.into_iter();

    let list = match args_iter.next() {
        Some(Value::List(l)) => Ok(l),
        Some(_) => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "cdr: argument is not a list".to_string(),
        }),
        None => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "cdr: must have an argument".to_string(),
        }),
    }?;

    if args_iter.next().is_some() {
        return Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "cdr: must have only one argument".to_string(),
        });
    }

    Ok(cdr(list.into()).unwrap_or(List::Empty).into())
}

fn func_cons(args: &List, _parent_ctx: &Context, source_info: &SourceInfo) -> Result<Value, Error> {
    let mut args_iter = args.into_iter();

    let head = match args_iter.next() {
        Some(v) => Ok(v),
        None => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "cons: must have two arguments".to_string(),
        }),
    }?;

    let tail = match args_iter.next() {
        Some(Value::List(l)) => Ok(l),
        Some(_) => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "cons: the second argument is not a list".to_string(),
        }),
        None => Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "cons: must a second argument".to_string(),
        }),
    }?;

    if args_iter.next().is_some() {
        return Err(Error::BadFuncArgs {
            source_info: source_info.clone(),
            msg: "cons: must have only two arguments".to_string(),
        });
    }

    Ok(cons(head, tail.into()).into())
}
