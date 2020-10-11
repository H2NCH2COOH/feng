use super::{Atom, Error, List, Value};
use std::io::Write;

pub fn print<W: Write>(out: &mut W, val: &Value) -> Result<(), Error> {
    match val {
        Value::Atom(atom) => print_atom(out, atom),
        Value::List(list) => print_list(out, list),
        Value::Lambda(lambda) => todo!(),
        Value::Function(_) => todo!(),
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
            List::EmptyList => break,
            List::Head { head, tail } => {
                if !first {
                    write!(out, " ")?;
                }
                first = false;
                print(out, head)?;
                ptr = tail;
            }
        }
    }
    write!(out, ")")?;

    Ok(())
}
