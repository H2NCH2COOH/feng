use super::error::Error;
use super::source::{Atom, List, SourceInfo, Value};
use std::rc::Rc;
use std::vec::Vec;

struct Source<S>
where
    S: Iterator<Item = Result<char, Error>>,
{
    name: Rc<str>,
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
            name: name.into(),
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
    Error::Syntax { source_info, msg }
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

    while let Some(v) = parse_value(&mut source)? {
        result.push(v);
    }

    Ok(result)
}

fn parse_value<S>(source: &mut Source<S>) -> Result<Option<Value>, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    let mut stack: Vec<List> = Vec::new();
    loop {
        match source.current() {
            Some('(') => {
                source.next()?;

                stack.push(List {
                    list: Rc::new(Vec::new()),
                    source_info: source.current_pos(),
                });
            }
            Some(')') => {
                source.next()?;

                match stack.pop() {
                    None => {
                        return Err(syntax_err(source, "Unexcpected ')'".to_string()));
                    }
                    Some(mut list) => {
                        Rc::get_mut(&mut list.list).unwrap().shrink_to_fit();
                        match stack.last_mut() {
                            None => {
                                return Ok(Some(Value::List(list)));
                            }
                            Some(parent) => {
                                Rc::get_mut(&mut parent.list)
                                    .unwrap()
                                    .push(Value::List(list));
                            }
                        }
                    }
                }
            }
            Some(c) => {
                // Skip leading whitespace
                if c.is_whitespace() {
                    source.next()?;
                    continue;
                }

                let atom = Value::Atom(parse_atom(source)?);
                match stack.last_mut() {
                    None => {
                        /*
                         * Just a single Atom not in a list.
                         */
                        return Ok(Some(atom));
                    }
                    Some(list) => {
                        Rc::get_mut(&mut list.list).unwrap().push(atom);
                    }
                }
            }
            None => {
                if stack.is_empty() {
                    return Ok(None);
                } else {
                    return Err(syntax_err(source, "Excepting ')' found EOF".to_string()));
                }
            }
        }
    }
}

fn parse_atom<S>(source: &mut Source<S>) -> Result<Atom, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    let mut name = String::new();
    let source_info = source.current_pos();

    loop {
        let c = match source.current() {
            Some('\\') => {
                source.next()?;
                source
                    .current()
                    .ok_or_else(|| syntax_err(source, "Expect character after '\\'".to_string()))
            }
            Some('(') => Err(syntax_err(source, "Invalid charactor '('".to_string())),
            Some(c) => {
                if c.is_whitespace() || c == ')' {
                    break;
                } else {
                    Ok(c)
                }
            }
            None => break,
        }?;

        name.push(c);
        source.next()?;
    }

    Ok(Atom {
        name: name.into(),
        source_info,
    })
}
