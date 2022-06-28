use super::source::SourceInfo;

#[derive(Debug)]
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
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}
