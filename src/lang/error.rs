#[derive(Debug)]
pub enum Error {
    IoErr(std::io::Error),
    Utf8Err([u8; 4]),
    SyntaxErr {
        lineno: u64,
        charno: u64,
        msg: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoErr(e) => write!(f, "Io Error: {}", e),
            Error::Utf8Err(e) => write!(f, "Invalid bytes as UTF-8: {:?}", e),
            Error::SyntaxErr {
                lineno,
                charno,
                msg,
            } => write!(f, "Syntax error: {}\nAt {}:{}", msg, lineno, charno),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoErr(e)
    }
}
