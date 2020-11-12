use super::SourceInfo;

#[derive(Debug)]
pub enum Error {
    IoErr(std::io::Error),
    Utf8Err([u8; 4]),
    SyntaxErr {
        source_info: SourceInfo,
        msg: String,
    },
    ValueErr {
        source_info: SourceInfo,
        msg: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoErr(e) => write!(f, "Io Error: {}", e),
            Error::Utf8Err(e) => write!(f, "Invalid bytes as UTF-8: {:?}", e),
            Error::SyntaxErr { source_info, msg } => write!(
                f,
                "Syntax error on {} at {}:{}:\n{}",
                source_info.name, source_info.lineno, source_info.charno, msg
            ),
            Error::ValueErr { source_info, msg } => write!(
                f,
                "Value error on {} at {}:{}:\n{}",
                source_info.name, source_info.lineno, source_info.charno, msg
            ),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoErr(e)
    }
}
