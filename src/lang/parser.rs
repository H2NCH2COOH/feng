use super::error::Error;
use super::Value;
use std::vec::Vec;

struct Source<S>
where
    S: Iterator<Item = Result<char, Error>>,
{
    lineno: u64,
    charno: u64,
    stream: S,
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
                return Ok(None);
            }
        };

        if c == '\n' {
            self.lineno += 1;
            self.charno = 0;
        } else {
            self.charno += 1;
        }

        Ok(Some(c))
    }

    fn current_pos(&self) -> (u64, u64) {
        (self.lineno, self.charno)
    }
}

pub fn parse<S>(stream: S) -> Result<Vec<Value>, Error>
where
    S: Iterator<Item = Result<char, Error>>,
{
    todo!()
}
