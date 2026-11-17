use std::fmt;

/// A line/column position within a YAML input string.
///
/// Positions are **1-indexed** (the first character of the input is line 1,
/// column 1).
#[derive(Debug)]
pub struct Location {
    line: usize,
    column: usize,
}

impl Location {
    /// Create a new `Location` with the given 1-indexed line and column.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// The 1-indexed line number.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The 1-indexed column number.
    pub fn column(&self) -> usize {
        self.column
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {} column {}", self.line, self.column)
    }
}

/// An error that occurred during YAML serialization or deserialization.
///
/// Parse errors carry a [`Location`] indicating where in the input the
/// problem was detected.
#[derive(Debug)]
pub struct Error {
    message: String,
    location: Option<Location>,
}

impl Error {
    /// Create an error with a message and no location.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
        }
    }

    /// Attach a [`Location`] to this error.
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// The location in the YAML input where the error was detected, if available.
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }

    pub(crate) fn scan(err: yaml_rust2::ScanError) -> Self {
        let marker = err.marker();
        Self {
            message: err.to_string(),
            location: Some(Location {
                line: marker.line() + 1,
                column: marker.col() + 1,
            }),
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

/// A `Result` alias where the error type is [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
