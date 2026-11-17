use crate::error::{Error, Result};
use crate::mapping::{Mapping, MappingVisitor};
use crate::number::Number;
use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, EnumAccess, IntoDeserializer, MapAccess,
    SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use serde::ser::{self, Serialize, Serializer};
use std::fmt::{self, Display};
use std::ops;
use yaml_rust2::Yaml;

/// A YAML sequence: an ordered list of [`Value`]s.
pub type Sequence = Vec<Value>;

/// Any valid YAML value.
///
/// A `Value` is a dynamically typed representation of a YAML node. It can be
/// produced by deserializing into `Value`, built by hand, or converted from
/// Rust primitives via `From`.
///
/// # Examples
///
/// ```rust
/// use serde_yaml_rust2::Value;
///
/// let v: Value = serde_yaml_rust2::from_str("a: [1, 2, 3]").unwrap();
/// assert_eq!(v["a"][1].as_i64(), Some(2));
/// assert!(v["missing"].is_null());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Value {
    ///     /// A YAML null (`null`, `~`, or an empty value). This is the default.
    #[default]
    Null,
    ///     /// A YAML boolean (`true` or `false`).
    Bool(bool),
    ///     /// A YAML number: an integer or a floating point value.
    Number(Number),
    ///     /// A YAML string.
    String(String),
    ///     /// A YAML sequence (list) of values.
    Sequence(Sequence),
    ///     /// A YAML mapping (key/value pairs) that preserves insertion order.
    Mapping(Mapping),
}

static NULL: Value = Value::Null;

fn de_err(msg: impl Display) -> Error {
    <Error as de::Error>::custom(msg)
}

fn ser_err(msg: impl Display) -> Error {
    <Error as ser::Error>::custom(msg)
}

impl Value {
    /// Index into a sequence or mapping, returning `None` if the index is out of
    /// bounds, the key is absent, or the value is not the right kind of container.
    ///
    /// Use a `usize` to index a sequence and a `&str`/`String` to look up a string
    /// key in a mapping.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::Value;
    ///
    /// let v: Value = serde_yaml_rust2::from_str("k: [10, 20]").unwrap();
    /// assert_eq!(v.get("k").and_then(|s| s.get(0)), Some(&Value::from(10)));
    /// assert_eq!(v.get("nope"), None);
    /// ```
    pub fn get<I: ValueIndex>(&self, index: I) -> Option<&Value> {
        index.index_into(self)
    }

    /// Like [`get`](Value::get), but returns a mutable reference.
    pub fn get_mut<I: ValueIndex>(&mut self, index: I) -> Option<&mut Value> {
        index.index_into_mut(self)
    }

    /// Returns `true` if the value is [`Value::Null`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::Value;
    ///
    /// assert!(Value::Null.is_null());
    /// assert!(!Value::Bool(false).is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns `true` if the value is a [`Value::Bool`].
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Returns `true` if the value is a [`Value::Number`].
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Returns `true` if the value is a [`Value::String`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::Value;
    ///
    /// assert!(Value::from("hi").is_string());
    /// assert!(!Value::from(1).is_string());
    /// ```
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Returns `true` if the value is a [`Value::Sequence`].
    pub fn is_sequence(&self) -> bool {
        matches!(self, Value::Sequence(_))
    }

    /// Returns `true` if the value is a [`Value::Mapping`].
    pub fn is_mapping(&self) -> bool {
        matches!(self, Value::Mapping(_))
    }

    /// Returns the boolean if the value is a [`Value::Bool`], otherwise `None`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the value as an `i64` if it is a number representable as one, otherwise `None`.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    /// Returns the value as a `u64` if it is a non-negative integer, otherwise `None`.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    /// Returns the value as an `f64` if it is a number, otherwise `None`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    /// Returns the string slice if the value is a [`Value::String`], otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::Value;
    ///
    /// assert_eq!(Value::from("hello").as_str(), Some("hello"));
    /// assert_eq!(Value::Null.as_str(), None);
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a reference to the sequence if the value is a [`Value::Sequence`], otherwise `None`.
    pub fn as_sequence(&self) -> Option<&Sequence> {
        match self {
            Value::Sequence(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a mutable reference to the sequence if the value is a [`Value::Sequence`], otherwise `None`.
    pub fn as_sequence_mut(&mut self) -> Option<&mut Sequence> {
        match self {
            Value::Sequence(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a reference to the mapping if the value is a [`Value::Mapping`], otherwise `None`.
    pub fn as_mapping(&self) -> Option<&Mapping> {
        match self {
            Value::Mapping(m) => Some(m),
            _ => None,
        }
    }

    /// Returns a mutable reference to the mapping if the value is a [`Value::Mapping`], otherwise `None`.
    pub fn as_mapping_mut(&mut self) -> Option<&mut Mapping> {
        match self {
            Value::Mapping(m) => Some(m),
            _ => None,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Sequence(_) => "sequence",
            Value::Mapping(_) => "mapping",
        }
    }

    fn unexpected(&self) -> Unexpected<'_> {
        match self {
            Value::Null => Unexpected::Unit,
            Value::Bool(b) => Unexpected::Bool(*b),
            Value::Number(n) => match (n.as_u64(), n.as_i64()) {
                (Some(u), _) => Unexpected::Unsigned(u),
                (None, Some(i)) => Unexpected::Signed(i),
                _ => Unexpected::Float(n.as_f64().unwrap_or(f64::NAN)),
            },
            Value::String(s) => Unexpected::Str(s),
            Value::Sequence(_) => Unexpected::Seq,
            Value::Mapping(_) => Unexpected::Map,
        }
    }
}

/// Converts a `yaml_rust2` node into a [`Value`]. `BadValue` and aliases become `Null`.
impl From<Yaml> for Value {
    fn from(yaml: Yaml) -> Self {
        match yaml {
            Yaml::Null | Yaml::BadValue | Yaml::Alias(_) => Value::Null,
            Yaml::Boolean(b) => Value::Bool(b),
            Yaml::Integer(i) => Value::Number(Number::from(i)),
            Yaml::Real(s) => Value::Number(Number::from_f64_str(&s)),
            Yaml::String(s) => Value::String(s),
            Yaml::Array(a) => Value::Sequence(a.into_iter().map(Value::from).collect()),
            Yaml::Hash(h) => Value::Mapping(Mapping::from_yaml_hash(h)),
        }
    }
}

/// Converts a [`Value`] into a `yaml_rust2` node.
impl From<Value> for Yaml {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Yaml::Null,
            Value::Bool(b) => Yaml::Boolean(b),
            Value::Number(n) => match n.as_i64() {
                Some(i) => Yaml::Integer(i),
                None => Yaml::Real(n.to_string()),
            },
            Value::String(s) => Yaml::String(s),
            Value::Sequence(s) => Yaml::Array(s.into_iter().map(Yaml::from).collect()),
            Value::Mapping(m) => Yaml::Hash(m.into_yaml_hash()),
        }
    }
}

/// Creates a [`Value::Bool`].
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

/// Creates a [`Value::String`].
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

/// Creates a [`Value::String`] by copying the slice.
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_owned())
    }
}

/// Creates a [`Value::Number`].
impl From<Number> for Value {
    fn from(v: Number) -> Self {
        Value::Number(v)
    }
}

/// Creates a [`Value::Sequence`].
impl From<Sequence> for Value {
    fn from(v: Sequence) -> Self {
        Value::Sequence(v)
    }
}

/// Creates a [`Value::Mapping`].
impl From<Mapping> for Value {
    fn from(v: Mapping) -> Self {
        Value::Mapping(v)
    }
}

macro_rules! from_number {
    ($($t:ty),*) => {$(
        /// Creates a [`Value::Number`] from a primitive numeric type.
        impl From<$t> for Value {
            fn from(v: $t) -> Self {
                Value::Number(Number::from(v))
            }
        }
    )*};
}

from_number!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// Collects an iterator of convertible items into a [`Value::Sequence`].
///
/// # Examples
///
/// ```rust
/// use serde_yaml_rust2::Value;
///
/// let v: Value = (1..=3).collect();
/// assert_eq!(v.as_sequence().unwrap().len(), 3);
/// ```
impl<T: Into<Value>> FromIterator<T> for Value {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Value::Sequence(iter.into_iter().map(Into::into).collect())
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for usize {}
    impl Sealed for str {}
    impl Sealed for String {}
    impl<T: ?Sized + Sealed> Sealed for &T {}
}

/// A type that can be used to index into a [`Value`].
///
/// Implemented for `usize` (sequence index) and `str`/`String` (mapping key),
/// plus references to those. The trait is sealed and cannot be implemented
/// outside this crate. It is used by [`Value::get`], [`Value::get_mut`] and the
/// `[]` operator.
pub trait ValueIndex: private::Sealed {
    #[doc(hidden)]
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value>;
    #[doc(hidden)]
    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value>;
    #[doc(hidden)]
    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value;
}

impl ValueIndex for usize {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match v {
            Value::Sequence(s) => s.get(*self),
            _ => None,
        }
    }

    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match v {
            Value::Sequence(s) => s.get_mut(*self),
            _ => None,
        }
    }

    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        match v {
            Value::Sequence(s) => {
                let len = s.len();
                s.get_mut(*self).unwrap_or_else(|| {
                    panic!(
                        "index {} out of bounds for sequence of length {}",
                        self, len
                    )
                })
            }
            other => panic!("cannot index into YAML {} with {}", other.type_name(), self),
        }
    }
}

impl ValueIndex for str {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match v {
            Value::Mapping(m) => m.get(&Value::String(self.to_owned())),
            _ => None,
        }
    }

    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match v {
            Value::Mapping(m) => m.get_mut(&Value::String(self.to_owned())),
            _ => None,
        }
    }

    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        if v.is_null() {
            *v = Value::Mapping(Mapping::new());
        }
        match v {
            Value::Mapping(m) => m
                .entry(Value::String(self.to_owned()))
                .or_insert(Value::Null),
            other => panic!("cannot access key {:?} in YAML {}", self, other.type_name()),
        }
    }
}

impl ValueIndex for String {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        self.as_str().index_into(v)
    }

    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        self.as_str().index_into_mut(v)
    }

    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        self.as_str().index_or_insert(v)
    }
}

impl<T: ?Sized + ValueIndex> ValueIndex for &T {
    fn index_into<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        (**self).index_into(v)
    }

    fn index_into_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        (**self).index_into_mut(v)
    }

    fn index_or_insert<'v>(&self, v: &'v mut Value) -> &'v mut Value {
        (**self).index_or_insert(v)
    }
}

/// Index into a [`Value`] with `[]`.
///
/// Never panics: a missing key, out-of-range index, or index into a value of
/// the wrong kind yields [`Value::Null`].
///
/// # Examples
///
/// ```rust
/// use serde_yaml_rust2::Value;
///
/// let v: Value = serde_yaml_rust2::from_str("a: {b: 1}").unwrap();
/// assert_eq!(v["a"]["b"].as_i64(), Some(1));
/// assert!(v["x"]["y"].is_null());
/// ```
impl<I: ValueIndex> ops::Index<I> for Value {
    type Output = Value;

    fn index(&self, index: I) -> &Value {
        index.index_into(self).unwrap_or(&NULL)
    }
}

/// Mutably index into a [`Value`] with `[]`.
///
/// # Panics
///
/// Panics if the index is out of bounds, the key is missing, or the value is
/// not a container of the matching kind.
impl<I: ValueIndex> ops::IndexMut<I> for Value {
    fn index_mut(&mut self, index: I) -> &mut Value {
        index.index_or_insert(self)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(n) => n.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            Value::Sequence(seq) => seq.serialize(serializer),
            Value::Mapping(map) => map.serialize(serializer),
        }
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any valid YAML value")
    }

    fn visit_bool<E>(self, v: bool) -> std::result::Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Value, E> {
        Ok(Value::Number(Number::from(v)))
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Value, E> {
        Ok(Value::Number(Number::from(v)))
    }

    fn visit_f64<E>(self, v: f64) -> std::result::Result<Value, E> {
        Ok(Value::Number(Number::from(v)))
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Value, E> {
        Ok(Value::String(v))
    }

    fn visit_unit<E>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> std::result::Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> std::result::Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(4096));
        while let Some(v) = seq.next_element()? {
            out.push(v);
        }
        Ok(Value::Sequence(out))
    }

    fn visit_map<A>(self, access: A) -> std::result::Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        MappingVisitor.visit_map(access).map(Value::Mapping)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = Value;

    fn into_deserializer(self) -> Value {
        self
    }
}

struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(v).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct MapDeserializer {
    iter: indexmap::map::IntoIter<Value, Value>,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                seed.deserialize(k).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(v) => seed.deserialize(v),
            None => Err(de_err("mapping value requested before key")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

fn visit_sequence<'de, V>(seq: Sequence, visitor: V) -> Result<V::Value>
where
    V: Visitor<'de>,
{
    let len = seq.len();
    let mut access = SeqDeserializer {
        iter: seq.into_iter(),
    };
    let out = visitor.visit_seq(&mut access)?;
    if access.iter.len() == 0 {
        Ok(out)
    } else {
        Err(de::Error::invalid_length(
            len,
            &"fewer elements in sequence",
        ))
    }
}

fn visit_mapping<'de, V>(map: Mapping, visitor: V) -> Result<V::Value>
where
    V: Visitor<'de>,
{
    let len = map.len();
    let mut access = MapDeserializer {
        iter: map.into_iter(),
        value: None,
    };
    let out = visitor.visit_map(&mut access)?;
    if access.iter.len() == 0 {
        Ok(out)
    } else {
        Err(de::Error::invalid_length(len, &"fewer elements in mapping"))
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(Value::String(self.variant))?;
        Ok((variant, VariantDeserializer { value: self.value }))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            None | Some(Value::Null) => Ok(()),
            Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"unit variant")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(v) => seed.deserialize(v),
            None => Err(de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Sequence(seq)) => visit_sequence(seq, visitor),
            Some(other) => Err(de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Mapping(map)) => visit_mapping(map, visitor),
            Some(Value::Sequence(seq)) => visit_sequence(seq, visitor),
            Some(other) => Err(de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Number(n) => n.deserialize_any(visitor),
            Value::String(s) => visitor.visit_string(s),
            Value::Sequence(seq) => visit_sequence(seq, visitor),
            Value::Mapping(map) => visit_mapping(map, visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            other => visitor.visit_some(other),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            Value::String(s) => (s, None),
            Value::Mapping(map) => {
                let mut iter = map.into_iter();
                let (key, value) =
                    match iter.next() {
                        Some(entry) => entry,
                        None => return Err(de_err(
                            "expected a mapping with a single key for enum, found an empty mapping",
                        )),
                    };
                if iter.next().is_some() {
                    return Err(de_err(
                        "expected a mapping with a single key for enum, found multiple keys",
                    ));
                }
                match key {
                    Value::String(s) => (s, Some(value)),
                    other => {
                        return Err(de::Error::invalid_type(
                            other.unexpected(),
                            &"string enum variant name",
                        ))
                    }
                }
            }
            other => {
                return Err(de::Error::invalid_type(
                    other.unexpected(),
                    &"string or single-key mapping for enum",
                ))
            }
        };
        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
        identifier ignored_any
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ValueSerializer;

impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Value> {
        if let Ok(v) = i64::try_from(v) {
            Ok(Value::from(v))
        } else if let Ok(v) = u64::try_from(v) {
            Ok(Value::from(v))
        } else {
            Err(ser_err("i128 value is out of range for a YAML number"))
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Value> {
        match u64::try_from(v) {
            Ok(v) => Ok(Value::from(v)),
            Err(_) => Err(ser_err("u128 value is out of range for a YAML number")),
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Value> {
        Ok(Value::from(v))
    }

    fn serialize_char(self, v: char) -> Result<Value> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        Ok(Value::Sequence(
            v.iter().copied().map(Value::from).collect(),
        ))
    }

    fn serialize_none(self) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        Ok(Value::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        let mut map = Mapping::with_capacity(1);
        map.insert(
            Value::String(variant.to_owned()),
            value.serialize(ValueSerializer)?,
        );
        Ok(Value::Mapping(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SerializeVec> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeVec> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SerializeVec> {
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
            name: variant,
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<SerializeMap> {
        Ok(SerializeMap {
            map: Mapping::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<SerializeMap> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeStructVariant> {
        Ok(SerializeStructVariant {
            name: variant,
            map: Mapping::with_capacity(len),
        })
    }
}

pub(crate) struct SerializeVec {
    vec: Vec<Value>,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Sequence(self.vec))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

pub(crate) struct SerializeTupleVariant {
    name: &'static str,
    vec: Vec<Value>,
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut map = Mapping::with_capacity(1);
        map.insert(
            Value::String(self.name.to_owned()),
            Value::Sequence(self.vec),
        );
        Ok(Value::Mapping(map))
    }
}

pub(crate) struct SerializeMap {
    map: Mapping,
    next_key: Option<Value>,
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(ValueSerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| ser_err("serialize_value called before serialize_key"))?;
        self.map.insert(key, value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.map.insert(
            key.serialize(ValueSerializer)?,
            value.serialize(ValueSerializer)?,
        );
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Mapping(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(
            Value::String(key.to_owned()),
            value.serialize(ValueSerializer)?,
        );
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Mapping(self.map))
    }
}

pub(crate) struct SerializeStructVariant {
    name: &'static str,
    map: Mapping,
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(
            Value::String(key.to_owned()),
            value.serialize(ValueSerializer)?,
        );
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut map = Mapping::with_capacity(1);
        map.insert(
            Value::String(self.name.to_owned()),
            Value::Mapping(self.map),
        );
        Ok(Value::Mapping(map))
    }
}
