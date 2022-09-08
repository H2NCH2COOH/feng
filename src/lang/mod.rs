use std::vec::Vec;

mod chars;
mod core;
mod error;
mod parser;
mod printer;
mod source;
#[cfg(test)]
mod tests;
mod value;

use error::Error;

pub fn parse<S>(name: &str, stream: &mut S) -> Result<Vec<source::Value>, Error>
where
    S: Iterator<Item = std::io::Result<u8>>,
{
    parser::parse(name, chars::Chars::new(stream))
}

pub fn parse_str(s: &str) -> Result<Vec<source::Value>, Error> {
    parser::parse(&format!("str:`{:.10}...'", s.trim()), s.chars().map(Ok))
}

pub fn print<W: std::io::Write>(out: &mut W, val: &value::Value) -> Result<(), Error> {
    printer::print(out, val)
}

pub fn println<W: std::io::Write>(out: &mut W, val: &value::Value) -> Result<(), Error> {
    printer::print(out, val)?;
    out.write_all(b"\n")?;
    Ok(())
}

pub fn eval_source(src: &[source::Value]) -> Result<value::Value, Error> {
    core::eval_source(src)
}
