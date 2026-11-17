use serde::ser::{self, Serialize};
use yaml_rust2::yaml::Hash;
use yaml_rust2::{Yaml, YamlEmitter};

use crate::error::{Error, Result};

/// Options for controlling YAML serialization output.
///
/// Use the builder methods to customise the output format, then pass the
/// options to [`to_string_with_options`] or [`to_writer_with_options`].
///
/// ```rust
/// use serde_yaml_rust2::SerializeOptions;
///
/// let opts = SerializeOptions::new()
///     .indent(4)
///     .document_start(true);
/// ```
#[derive(Clone, Debug)]
pub struct SerializeOptions {
    indent: usize,
    flow_sequences: bool,
    flow_mappings: bool,
    document_start: bool,
}

impl SerializeOptions {
    /// Create options with the default settings.
    ///
    /// Defaults: 2-space indent, block style (no flow), no document-start
    /// marker.
    pub fn new() -> Self {
        Self {
            indent: 2,
            flow_sequences: false,
            flow_mappings: false,
            document_start: false,
        }
    }

    /// Set the indentation width in spaces (default: 2).
    pub fn indent(mut self, n: usize) -> Self {
        self.indent = n;
        self
    }

    /// Emit short, all-scalar sequences in flow style (`[1, 2, 3]`).
    ///
    /// **Note:** yaml-rust2 does not expose a flow-style toggle yet, so this
    /// option is accepted and stored but has no effect on the output at this
    /// time.
    pub fn flow_sequences(mut self, yes: bool) -> Self {
        self.flow_sequences = yes;
        self
    }

    /// Emit short, all-scalar mappings in flow style (`{a: 1, b: 2}`).
    ///
    /// **Note:** yaml-rust2 does not expose a flow-style toggle yet, so this
    /// option is accepted and stored but has no effect on the output at this
    /// time.
    pub fn flow_mappings(mut self, yes: bool) -> Self {
        self.flow_mappings = yes;
        self
    }

    /// Emit a `---` document-start marker (default: `false`).
    pub fn document_start(mut self, yes: bool) -> Self {
        self.document_start = yes;
        self
    }
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self::new()
    }
}

fn reindent(yaml: &str, target: usize) -> String {
    const BASE: usize = 2;
    if target == BASE {
        return yaml.to_owned();
    }
    let mut result = String::with_capacity(yaml.len());
    for (i, line) in yaml.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let spaces = line.len() - line.trim_start().len();
        if spaces == 0 {
            result.push_str(line);
        } else {
            let level = spaces / BASE;
            let remainder = spaces % BASE;
            for _ in 0..(level * target + remainder) {
                result.push(' ');
            }
            result.push_str(line.trim_start());
        }
    }
    result
}

/// Serialize `T` into a YAML string without a leading `---` document marker.
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    to_string_with_options(value, &SerializeOptions::default())
}

/// Serialize `T` into a YAML string using the given [`SerializeOptions`].
pub fn to_string_with_options<T: ?Sized + Serialize>(
    value: &T,
    options: &SerializeOptions,
) -> Result<String> {
    let yaml = value.serialize(Serializer)?;
    let mut out = String::new();
    YamlEmitter::new(&mut out)
        .dump(&yaml)
        .map_err(Error::emit)?;

    let body = if options.document_start {
        out.as_str()
    } else {
        let stripped = out.strip_prefix("---").unwrap_or(&out);
        stripped
            .strip_prefix('\n')
            .or_else(|| stripped.strip_prefix(' '))
            .unwrap_or(stripped)
    };

    let mut result = reindent(body, options.indent);
    if !result.ends_with('\n') {
        result.push('\n');
    }
    Ok(result)
}

/// Serialize `T` as YAML and write it into `writer`.
pub fn to_writer<W: std::io::Write, T: ?Sized + Serialize>(writer: W, value: &T) -> Result<()> {
    to_writer_with_options(writer, value, &SerializeOptions::default())
}

/// Serialize `T` as YAML into `writer` using the given [`SerializeOptions`].
pub fn to_writer_with_options<W: std::io::Write, T: ?Sized + Serialize>(
    mut writer: W,
    value: &T,
    options: &SerializeOptions,
) -> Result<()> {
    let text = to_string_with_options(value, options)?;
    writer
        .write_all(text.as_bytes())
        .map_err(|e| Error::new(e.to_string()))
}

/// A serde [`Serializer`](serde::Serializer) that produces a yaml-rust2 `Yaml` value.
///
/// Normally used indirectly through [`to_string`] or [`to_writer`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Serializer;

fn real_from_f64(v: f64) -> Yaml {
    if v.is_nan() {
        Yaml::Real(".nan".to_owned())
    } else if v.is_infinite() {
        Yaml::Real(if v > 0.0 { ".inf" } else { "-.inf" }.to_owned())
    } else {
        Yaml::Real(ryu::Buffer::new().format_finite(v).to_owned())
    }
}

fn real_from_f32(v: f32) -> Yaml {
    if v.is_nan() {
        Yaml::Real(".nan".to_owned())
    } else if v.is_infinite() {
        Yaml::Real(if v > 0.0 { ".inf" } else { "-.inf" }.to_owned())
    } else {
        Yaml::Real(ryu::Buffer::new().format_finite(v).to_owned())
    }
}

fn single_entry(key: &str, value: Yaml) -> Yaml {
    let mut map = Hash::new();
    map.insert(Yaml::String(key.to_owned()), value);
    Yaml::Hash(map)
}

impl ser::Serializer for Serializer {
    type Ok = Yaml;
    type Error = Error;

    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeSeq;
    type SerializeTupleStruct = SerializeSeq;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Yaml> {
        Ok(Yaml::Boolean(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Yaml> {
        Ok(Yaml::Integer(i64::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Yaml> {
        Ok(Yaml::Integer(i64::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Yaml> {
        Ok(Yaml::Integer(i64::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Yaml> {
        Ok(Yaml::Integer(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Yaml> {
        Ok(match i64::try_from(v) {
            Ok(n) => Yaml::Integer(n),
            Err(_) => Yaml::Real(itoa::Buffer::new().format(v).to_owned()),
        })
    }

    fn serialize_u8(self, v: u8) -> Result<Yaml> {
        Ok(Yaml::Integer(i64::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Yaml> {
        Ok(Yaml::Integer(i64::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Yaml> {
        Ok(Yaml::Integer(i64::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Yaml> {
        Ok(match i64::try_from(v) {
            Ok(n) => Yaml::Integer(n),
            Err(_) => Yaml::Real(itoa::Buffer::new().format(v).to_owned()),
        })
    }

    fn serialize_u128(self, v: u128) -> Result<Yaml> {
        Ok(match i64::try_from(v) {
            Ok(n) => Yaml::Integer(n),
            Err(_) => Yaml::Real(itoa::Buffer::new().format(v).to_owned()),
        })
    }

    fn serialize_f32(self, v: f32) -> Result<Yaml> {
        Ok(real_from_f32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Yaml> {
        Ok(real_from_f64(v))
    }

    fn serialize_char(self, v: char) -> Result<Yaml> {
        Ok(Yaml::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Yaml> {
        Ok(Yaml::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Yaml> {
        Ok(Yaml::Array(
            v.iter().map(|&b| Yaml::Integer(i64::from(b))).collect(),
        ))
    }

    fn serialize_none(self) -> Result<Yaml> {
        Ok(Yaml::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Yaml> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Yaml> {
        Ok(Yaml::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Yaml> {
        Ok(Yaml::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Yaml> {
        Ok(Yaml::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Yaml> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Yaml> {
        Ok(single_entry(variant, value.serialize(Serializer)?))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SerializeSeq> {
        Ok(SerializeSeq {
            items: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeSeq> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SerializeSeq> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            variant,
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<SerializeMap> {
        Ok(SerializeMap {
            map: Hash::new(),
            pending_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<SerializeStruct> {
        Ok(SerializeStruct { map: Hash::new() })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SerializeStructVariant> {
        Ok(SerializeStructVariant {
            variant,
            map: Hash::new(),
        })
    }
}

pub struct SerializeSeq {
    items: Vec<Yaml>,
}

impl SerializeSeq {
    fn push<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }
}

impl ser::SerializeSeq for SerializeSeq {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.push(value)
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Array(self.items))
    }
}

impl ser::SerializeTuple for SerializeSeq {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.push(value)
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Array(self.items))
    }
}

impl ser::SerializeTupleStruct for SerializeSeq {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.push(value)
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Array(self.items))
    }
}

pub struct SerializeTupleVariant {
    variant: &'static str,
    items: Vec<Yaml>,
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(single_entry(self.variant, Yaml::Array(self.items)))
    }
}

pub struct SerializeMap {
    map: Hash,
    pending_key: Option<Yaml>,
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.pending_key = Some(key.serialize(Serializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| Error::new("serialize_value called before serialize_key"))?;
        self.map.insert(key, value.serialize(Serializer)?);
        Ok(())
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<()> {
        let key = key.serialize(Serializer)?;
        let value = value.serialize(Serializer)?;
        self.map.insert(key, value);
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Hash(self.map))
    }
}

pub struct SerializeStruct {
    map: Hash,
}

impl ser::SerializeStruct for SerializeStruct {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.map
            .insert(Yaml::String(key.to_owned()), value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(Yaml::Hash(self.map))
    }
}

pub struct SerializeStructVariant {
    variant: &'static str,
    map: Hash,
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Yaml;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.map
            .insert(Yaml::String(key.to_owned()), value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Yaml> {
        Ok(single_entry(self.variant, Yaml::Hash(self.map)))
    }
}
