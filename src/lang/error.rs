pub enum Error {
    IoErr(std::io::Error),
    Utf8Err([u8; 4]),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoErr(e) => write!(f, "Io Error: {}", e),
            Error::Utf8Err(e) => write!(f, "Invalid bytes as UTF-8: {:?}", e),
        }
    }
}
