use super::error::Error;
use super::source;
use super::value::{ArgList, Atom, Fexpr, Function, List, Value};
use std::io::Write;

pub fn print<W: Write>(out: &mut W, val: &Value) -> Result<(), Error> {
    match val {
        Value::Atom(atom) => print_atom(out, atom),
        Value::List(list) => print_list(out, list),
        Value::SourceAtom(source_atom) => print_source_atom(out, source_atom),
        Value::SourceList(source_list) => print_source_list(out, source_list),
        Value::Fexpr(fexpr) => print_fexpr(out, fexpr),
        Value::Function(func) => print_function(out, func),
    }
}

fn print_atom<W: Write>(out: &mut W, atom: &Atom) -> Result<(), Error> {
    write!(out, "{}", atom.name)?;
    Ok(())
}

fn print_list<W: Write>(out: &mut W, list: &List) -> Result<(), Error> {
    write!(out, "(")?;

    let mut first = true;
    let mut ptr = list;
    loop {
        match ptr {
            List::Empty => break,
            List::Head(head) => {
                if !first {
                    write!(out, " ")?;
                }
                first = false;
                print(out, &head.val)?;
                ptr = &head.tail;
            }
        }
    }
    write!(out, ")")?;
    Ok(())
}

fn print_source_atom<W: Write>(out: &mut W, atom: &source::Atom) -> Result<(), Error> {
    write!(out, "{}", atom.name)?;
    Ok(())
}

fn print_source_list<W: Write>(out: &mut W, list: &source::List) -> Result<(), Error> {
    write!(out, "(")?;
    if !list.list.is_empty() {
        fn print_source_value<W: Write>(out: &mut W, val: &source::Value) -> Result<(), Error> {
            match val {
                source::Value::Atom(atom) => print_source_atom(out, atom),
                source::Value::List(list) => print_source_list(out, list),
            }
        }

        print_source_value(out, &list.list[0])?;
        for v in &list.list[1..] {
            write!(out, " ")?;
            print_source_value(out, v)?;
        }
    }
    write!(out, ")")?;
    Ok(())
}

fn print_fexpr<W: Write>(out: &mut W, fexpr: &Fexpr) -> Result<(), Error> {
    write!(out, "(fexpr! ")?;
    match &fexpr.arg_list {
        ArgList::Args(list) => {
            if list.is_empty() {
                write!(out, "() ")?;
            } else {
                write!(out, "(")?;
                write!(out, "{}", list[0].name)?;
                for arg in &list[1..] {
                    write!(out, " {}", arg.name)?;
                }
                write!(out, ") ")?
            }
        }
        ArgList::Vargs(atom) => write!(out, "{} ", atom.name)?,
    };
    print_list(out, &fexpr.body);
    write!(out, ")")?;
    Ok(())
}

fn print_function<W: Write>(out: &mut W, func: &Function) -> Result<(), Error> {
    todo!()
}
