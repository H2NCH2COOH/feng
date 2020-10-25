use super::{Atom, Error, List, SourceInfo, Value};
use std::rc::Rc;
use std::vec::Vec;

struct Source<S>
where
    S: Iterator<Item = Result<char, Error>>,
{
    name: Rc<String>,
    lineno: usize,
    charno: usize,
    stream: S,
    current: Option<char>,
}

impl<S> Source<S>
where
    S: Iterator<Item = Result<char, Error>>,
{
    fn new(name: &str, stream: S) -> Self {
        Self {
            name: Rc::new(name.to_owned()),
            lineno: 1,
            charno: 0,
            stream,
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

    fn current_pos(&self) -> SourceInfo {
        SourceInfo {
            name: self.name.clone(),
            lineno: self.lineno,
            charno: self.charno,
        }
    }
}

fn syntax_err<S>(source: &Source<S>, msg: String) -> Error
where
    S: Iterator<Item = Result<char, Error>>,
{
    let source_info = source.current_pos();
    Error::SyntaxErr { source_info, msg }
}

pub fn parse<S>(name: &str, stream: S) -> Result<Vec<Value>, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    let mut source = Source::new(name, stream);
    let mut result = Vec::new();

    // Handle empty source and make sure source.current() is valid
    if source.next()?.is_none() {
        return Ok(result);
    }

    while let Some(v) = parse_value(&mut source, false)? {
        result.push(v);
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
    let source_info = source.current_pos();
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

    Ok(Atom {
        name,
        source_info: Some(source_info),
    })
}

fn parse_list<S>(source: &mut Source<S>) -> Result<Rc<List>, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    assert!(source.current().unwrap() == '(');

    let source_info = source.current_pos();

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

    let mut head = Rc::new(List::EmptyList { source_info: None });
    for v in buf.iter().rev() {
        head = Rc::new(List::Head {
            head: v.clone(),
            tail: head,
            source_info: None,
        })
    }

    match Rc::get_mut(&mut head).unwrap() {
        List::EmptyList { source_info: si } => *si = Some(source_info),
        List::Head {
            head: _,
            tail: _,
            source_info: si,
        } => *si = Some(source_info),
    };

    Ok(head)
}
