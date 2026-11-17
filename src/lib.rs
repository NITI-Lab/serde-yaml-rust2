#![forbid(unsafe_code)]

mod de;
mod error;
mod mapping;
mod number;
pub(crate) mod path;
mod ser;
mod value;

pub use de::Deserializer;
pub use error::{Error, Location, Result};
pub use mapping::Mapping;
pub use number::Number;
pub use ser::Serializer;
pub use value::{Sequence, Value};

pub fn from_str<T>(s: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    de::from_str(s)
}

pub fn from_slice<T>(v: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let s = std::str::from_utf8(v).map_err(|e| Error::new(e.to_string()))?;
    from_str(s)
}

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

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: serde::Serialize,
{
    ser::to_string(value)
}

pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: serde::Serialize,
{
    ser::to_writer(writer, value)
}

pub fn from_value<T>(value: Value) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    T::deserialize(value)
}

pub fn to_value<T>(value: &T) -> Result<Value>
where
    T: serde::Serialize,
{
    value.serialize(value::ValueSerializer)
}
