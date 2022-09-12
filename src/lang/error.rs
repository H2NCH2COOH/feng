use super::source::SourceInfo;
use super::value::{Atom, List, Value};
use std::collections::HashMap;

pub enum Error {
    Io(std::io::Error),
    Utf8([u8; 4]),
    Syntax {
        source_info: SourceInfo,
        msg: String,
    },
    NoUpCtx {
        source_info: SourceInfo,
    },
    CantEval {
        source_info: SourceInfo,
        val: Value,
    },
    CantCall {
        source_info: SourceInfo,
        val: Value,
    },
    BadArgsNum {
        source_info: SourceInfo,
        expected: usize,
        found: usize,
    },
    BadFuncArgs {
        source_info: SourceInfo,
        msg: String,
    },
    AssertError {
        source_info: SourceInfo,
        msg: String,
    },
    Redefinition {
        source_info: SourceInfo,
        key: Atom,
        old_val: Value,
        new_val: Value,
    },
    TailRecursionRequest {
        source_info: SourceInfo,
        args: List,
        map: HashMap<Atom, Value>,
    },
    CantTailRecursion {
        source_info: SourceInfo,
        msg: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "Io Error: {}", e),
            Error::Utf8(e) => write!(f, "Invalid bytes as UTF-8: {:?}", e),
            Error::Syntax { source_info, msg } => {
                write!(f, "Syntax error: {}\n\tAt {}", msg, source_info)
            }
            Error::NoUpCtx { source_info } => write!(f, "Can't go upwards\n\tAt {}", source_info),
            Error::CantEval { source_info, val } => {
                write!(f, "Can't eval value: {}\n\tAt {}", val, source_info)
            }
            Error::CantCall { source_info, val } => {
                write!(f, "Can't call value: {}\n\tAt {}", val, source_info)
            }
            Error::BadArgsNum {
                source_info,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Bad number of arguments, expected {}, found {}\n\tAt {}",
                    expected, found, source_info
                )
            }
            Error::BadFuncArgs { source_info, msg } => {
                write!(f, "Bad arguments: {}\n\tAt {}", msg, source_info)
            }
            Error::AssertError { source_info, msg } => {
                write!(f, "Assert failed with: {}\n\tAt {}", msg, source_info)
            }
            Error::Redefinition {
                source_info,
                key,
                old_val,
                new_val,
            } => {
                write!(
                    f,
                    "Can't redefine {} from {} to {}\n\tAt {}",
                    key, old_val, new_val, source_info
                )
            }
            Error::TailRecursionRequest {
                source_info,
                args: _,
                map: _,
            } => {
                write!(
                    f,
                    "An unhandled tail recursion (this error shouldn't be seen)\n\tAt {}",
                    source_info
                )
            }
            Error::CantTailRecursion { source_info, msg } => {
                write!(f, "Can't tail recursion: {}\n\tAt {}", msg, source_info)
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}
