use std::fmt;

#[derive(Debug)]
pub struct Location {
    line: usize,
    column: usize,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {} column {}", self.line, self.column)
    }
}

#[derive(Debug)]
pub struct Error {
    message: String,
    location: Option<Location>,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
        }
    }

    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }

    pub(crate) fn scan(err: yaml_rust2::ScanError) -> Self {
        let msg = err.to_string();
        Self {
            message: msg,
            location: None,
        }
    }

    pub(crate) fn emit(err: yaml_rust2::EmitError) -> Self {
        Self::new(err.to_string())
    }

    pub(crate) fn bad_value(path: &crate::path::Path) -> Self {
        Self::new(format!("bad YAML value at {path}"))
    }

    pub(crate) fn end_of_stream() -> Self {
        Self::new("EOF while parsing a value")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(loc) = &self.location {
            write!(f, "{}: {}", loc, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::new(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::new(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
