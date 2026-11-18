//! YAML serialization and deserialization built on
//! [yaml-rust2](https://github.com/Ethiraric/yaml-rust2).
//!
//! This crate provides a [serde](https://serde.rs/) integration for YAML 1.2,
//! using a safe, pure-Rust parser with no `unsafe` code.
//!
//! # Examples
//!
//! Deserialize a struct from YAML:
//!
//! ```rust
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Config {
//!     name: String,
//!     debug: bool,
//! }
//!
//! let yaml = "name: app\ndebug: true\n";
//! let config: Config = serde_yaml_rust2::from_str(yaml).unwrap();
//! assert_eq!(config.name, "app");
//! ```
//!
//! Serialize a struct to YAML:
//!
//! ```rust
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Point { x: f64, y: f64 }
//!
//! let yaml = serde_yaml_rust2::to_string(&Point { x: 1.0, y: 2.0 }).unwrap();
//! assert!(yaml.contains("x:"));
//! ```

#![forbid(unsafe_code)]

mod de;
mod emit;
mod error;
mod load;
mod mapping;
mod number;
pub(crate) mod path;
mod ser;
mod value;

pub use de::Deserializer;
pub use error::{Error, Location, Result};
pub use mapping::Mapping;
pub use number::Number;
pub use ser::{SerializeOptions, Serializer};
pub use value::{Sequence, Value};

/// Deserialize a `T` from a YAML string.
///
/// This parses the first YAML document in the string. To deserialize all
/// documents, use [`from_str_multidoc`].
pub fn from_str<T>(s: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    de::from_str(s)
}

/// Deserialize every `---`-separated YAML document in a string into a `Vec<T>`.
///
/// Returns an empty `Vec` for an empty string.
///
/// # Examples
///
/// ```rust
/// use serde_yaml_rust2::Value;
///
/// let yaml = "---\na: 1\n---\nb: 2\n";
/// let docs: Vec<Value> = serde_yaml_rust2::from_str_multidoc(yaml).unwrap();
/// assert_eq!(docs.len(), 2);
/// ```
pub fn from_str_multidoc<T>(s: &str) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let deserializers = Deserializer::from_str_documents(s)?;
    deserializers
        .into_iter()
        .map(|de| T::deserialize(de))
        .collect()
}

/// Deserialize a `T` from a UTF-8 byte slice containing YAML.
pub fn from_slice<T>(v: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let s = std::str::from_utf8(v).map_err(|e| Error::new(e.to_string()))?;
    from_str(s)
}

/// Deserialize a `T` from any [`std::io::Read`] source containing YAML.
pub fn from_reader<R, T>(mut rdr: R) -> Result<T>
where
    R: std::io::Read,
    T: serde::de::DeserializeOwned,
{
    let mut s = String::new();
    rdr.read_to_string(&mut s)
        .map_err(|e| Error::new(e.to_string()))?;
    from_str(&s)
}

/// Serialize `T` into a YAML string.
///
/// The output does not include a `---` document start marker.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + serde::Serialize,
{
    ser::to_string(value)
}

/// Serialize `T` as YAML into any [`std::io::Write`] destination.
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: ?Sized + serde::Serialize,
{
    ser::to_writer(writer, value)
}

/// Serialize `T` into a YAML string using the given [`SerializeOptions`].
pub fn to_string_with_options<T>(value: &T, options: &SerializeOptions) -> Result<String>
where
    T: ?Sized + serde::Serialize,
{
    ser::to_string_with_options(value, options)
}

/// Serialize `T` as YAML into `writer` using the given [`SerializeOptions`].
pub fn to_writer_with_options<W, T>(writer: W, value: &T, options: &SerializeOptions) -> Result<()>
where
    W: std::io::Write,
    T: ?Sized + serde::Serialize,
{
    ser::to_writer_with_options(writer, value, options)
}

/// Convert a [`Value`] into a typed value `T`.
pub fn from_value<T>(value: Value) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    T::deserialize(value)
}

/// Convert a typed value `T` into a [`Value`].
pub fn to_value<T>(value: &T) -> Result<Value>
where
    T: ?Sized + serde::Serialize,
{
    value.serialize(value::ValueSerializer)
}
