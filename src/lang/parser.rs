use super::Error;
use super::{Atom, List, Value};
use std::rc::Rc;
use std::vec::Vec;

struct Source<S>
where
    S: Iterator<Item = Result<char, Error>>,
{
    lineno: u64,
    charno: u64,
    stream: S,
    current: Option<char>,
}

impl<S> Source<S>
where
    S: Iterator<Item = Result<char, Error>>,
{
    fn new(mut stream: S) -> Self {
        Self {
            lineno: 1,
            charno: 0,
            stream: stream,
            current: None,
        }
    }

    fn next(&mut self) -> Result<Option<char>, Error> {
        let rst = self.stream.next();

        let c = match rst {
            Some(Ok(c)) => c,
            Some(Err(e)) => {
                return Err(e);
            }
            None => {
                self.current = None;
                return Ok(None);
            }
        };

        if c == '\n' {
            self.lineno += 1;
            self.charno = 0;
        } else {
            self.charno += 1;
        }

        self.current = Some(c);
        Ok(self.current)
    }

    fn current(&self) -> Option<char> {
        self.current
    }

    fn current_pos(&self) -> (u64, u64) {
        (self.lineno, self.charno)
    }
}

fn syntax_err<S>(source: &Source<S>, msg: String) -> Error
where
    S: Iterator<Item = Result<char, Error>>,
{
    let (lineno, charno) = source.current_pos();
    Error::SyntaxErr {
        lineno: lineno,
        charno: charno,
        msg: msg,
    }
}

pub fn parse<S>(stream: S) -> Result<Vec<Value>, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    let mut source = Source::new(stream);
    let mut result = Vec::new();

    // Handle empty source and make sure source.current() is valid
    if source.next()?.is_none() {
        return Ok(result);
    }

    loop {
        match parse_value(&mut source, false)? {
            Some(v) => result.push(v),
            None => break,
        }
    }

    Ok(result)
}

fn parse_value<S>(source: &mut Source<S>, within_list: bool) -> Result<Option<Value>, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    // Skip leading whitespace
    loop {
        let cur = match source.current() {
            Some(c) => c,
            None => {
                return Ok(None);
            }
        };
        if !cur.is_whitespace() {
            break;
        }
        source.next()?;
    }

    match source.current() {
        Some('(') => Ok(Some(Value::List(parse_list(source)?))),
        Some(')') => {
            if within_list {
                Ok(None)
            } else {
                Err(syntax_err(source, format!("Unexcpected ')'")))
            }
        }
        Some(_) => Ok(Some(Value::Atom(parse_atom(source)?))),
        None => unreachable!(),
    }
}

fn parse_atom<S>(source: &mut Source<S>) -> Result<Atom, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    let mut name = String::new();
    loop {
        match source.current() {
            Some(c) => {
                if c.is_whitespace() || c == ')' {
                    break;
                } else if c == '(' {
                    return Err(syntax_err(source, format!("Invalid charactor '('")));
                } else {
                    name.push(c);
                }
            }
            None => break,
        }
        source.next()?;
    }

    Ok(Atom { name: name })
}

fn parse_list<S>(source: &mut Source<S>) -> Result<Rc<List>, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    assert!(source.current().unwrap() == '(');

    source.next()?; // Skip '('

    let mut buf = Vec::new();
    loop {
        match parse_value(source, true)? {
            Some(v) => {
                buf.push(v);
            }
            None => match source.current() {
                Some(')') => break,
                Some(_) => unreachable!(),
                None => {
                    return Err(syntax_err(source, format!("Excepting ')' found EOF")));
                }
            },
        }
    }

    source.next()?; // Skip ')'

    let mut head = Rc::new(List::EmptyList);
    for v in buf {
        head = Rc::new(List::Head {
            head: v,
            tail: head,
        })
    }

    Ok(head)
}