use std::collections::HashMap;
use std::rc::Rc;
use std::vec::Vec;

mod chars;
mod error;
mod parser;
mod printer;
mod source;
mod value;

use error::Error;

pub fn parse<S>(name: &str, stream: &mut S) -> Result<Vec<source::Value>, Error>
where
    S: Iterator<Item = std::io::Result<u8>>,
{
    parser::parse(name, chars::Chars::new(stream))
}

pub fn print<W: std::io::Write>(out: &mut W, val: &value::Value) -> Result<(), Error> {
    printer::print(out, val)
}
