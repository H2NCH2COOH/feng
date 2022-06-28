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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "Io Error: {}", e),
            Error::Utf8(e) => write!(f, "Invalid bytes as UTF-8: {:?}", e),
            Error::Syntax { source_info, msg } => write!(
                f,
                "Syntax error at {}:{}:{}:\n{}",
                source_info.name, source_info.lineno, source_info.charno, msg
            ),
            Error::NoUpCtx { source_info } => write!(
                f,
                "Can't go upwards at {}:{}:{}",
                source_info.name, source_info.lineno, source_info.charno
            ),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}
