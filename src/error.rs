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
/// problem was detected. Deserialization errors may also carry a YAML path
/// (e.g. `.foo.bar[0]`) showing where in the document structure the problem
/// occurred.
#[derive(Debug)]
pub struct Error {
    message: String,
    location: Option<Location>,
    path: Option<String>,
}

impl Error {
    /// Create an error with a message and no location or path.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
            path: None,
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

    /// The YAML path where the error occurred (e.g. `.foo.bar[0]`), if available.
    ///
    /// The path describes the position within the document's structure,
    /// while [`location`](Self::location) describes the byte position in the
    /// input string.
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Attach a YAML path to this error, unless it already has one.
    pub(crate) fn with_path(mut self, path: &crate::path::Path) -> Self {
        if self.path.is_none() {
            self.path = Some(path.to_string());
        }
        self
    }

    pub(crate) fn scan(err: yaml_rust2::ScanError) -> Self {
        let marker = err.marker();
        Self {
            message: err.info().to_owned(),
            // yaml-rust2 lines are already 1-based; columns are 0-based.
            location: Some(Location {
                line: marker.line(),
                column: marker.col() + 1,
            }),
            path: None,
        }
    }

    pub(crate) fn bad_value(path: &crate::path::Path) -> Self {
        Self::new(format!("bad YAML value at {path}"))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.location, &self.path) {
            (Some(loc), Some(path)) => write!(f, "{}, at {}: {}", loc, path, self.message),
            (Some(loc), None) => write!(f, "{}: {}", loc, self.message),
            (None, Some(path)) => write!(f, "at {}: {}", path, self.message),
            (None, None) => write!(f, "{}", self.message),
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
