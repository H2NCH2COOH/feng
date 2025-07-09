use super::source::SourceInfo;

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
        val: super::value::Value,
    },
    CantCall {
        source_info: SourceInfo,
        val: super::value::Value,
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
    AssertFailed {
        source_info: SourceInfo,
        msg: String,
    },
    Redefinition {
        source_info: SourceInfo,
        key: super::value::Atom,
        old_val: Box<super::value::Value>,
        new_val: Box<super::value::Value>,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "Io Error: {e}"),
            Error::Utf8(e) => write!(f, "Invalid bytes as UTF-8: {e:?}"),
            Error::Syntax { source_info, msg } => {
                write!(f, "Syntax error: {msg}\n\tAt {source_info}")
            }
            Error::NoUpCtx { source_info } => write!(f, "Can't go upwards\n\tAt {source_info}"),
            Error::CantEval { source_info, val } => {
                write!(f, "Can't eval value: {val}\n\tAt {source_info}")
            }
            Error::CantCall { source_info, val } => {
                write!(f, "Can't call value: {val}\n\tAt {source_info}")
            }
            Error::BadArgsNum {
                source_info,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Bad number of arguments, expected {expected}, found {found}\n\tAt {source_info}"
                )
            }
            Error::BadFuncArgs { source_info, msg } => {
                write!(f, "Bad arguments: {msg}\n\tAt {source_info}")
            }
            Error::AssertFailed { source_info, msg } => {
                write!(f, "Assert failed with: {msg}\n\tAt {source_info}")
            }
            Error::Redefinition {
                source_info,
                key,
                old_val,
                new_val,
            } => {
                write!(
                    f,
                    "Can't redefine {key} from {old_val} to {new_val}\n\tAt {source_info}"
                )
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}
