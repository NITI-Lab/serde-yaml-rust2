use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, Error as _, IntoDeserializer, MapAccess,
    SeqAccess, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use yaml_rust2::Yaml;

use crate::error::{Error, Result};
use crate::load::{load_from_str, Node};
use crate::path::Path;

/// Deserialize a `T` from a YAML string.
///
/// Only the first document in the string is used. For multi-document
/// input see [`Deserializer::from_str_documents`]. Input without any
/// document (empty, or only comments) reads as `null`.
pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T> {
    let docs = load_from_str(s).map_err(Error::scan)?;
    let doc = docs.into_iter().next().unwrap_or(Node::Null);
    T::deserialize(Deserializer::child(doc, Path::new()))
}

/// A serde [`Deserializer`](serde::Deserializer) backed by a YAML node tree.
///
/// Construct one from a [`Yaml`] node with [`Deserializer::new`], or parse
/// multiple documents with [`Deserializer::from_str_documents`].
pub struct Deserializer {
    yaml: Node,
    path: Path,
}

impl Deserializer {
    /// Create a deserializer from a pre-parsed [`Yaml`] node.
    pub fn new(yaml: Yaml) -> Self {
        Self {
            yaml: Node::from(yaml),
            path: Path::new(),
        }
    }

    fn child(yaml: Node, path: Path) -> Self {
        Self { yaml, path }
    }

    /// Create one `Deserializer` per YAML document in the string.
    ///
    /// Documents are separated by `---`. An empty string yields no documents.
    pub fn from_str_documents(s: &str) -> Result<Vec<Deserializer>> {
        let docs = load_from_str(s).map_err(Error::scan)?;
        Ok(docs
            .into_iter()
            .map(|yaml| Deserializer {
                yaml,
                path: Path::new(),
            })
            .collect())
    }
}

fn parse_real(s: &str) -> Option<f64> {
    match s.to_ascii_lowercase().as_str() {
        ".inf" | "+.inf" => Some(f64::INFINITY),
        "-.inf" => Some(f64::NEG_INFINITY),
        ".nan" => Some(f64::NAN),
        other => other.parse().ok(),
    }
}

fn key_label(key: &Node) -> String {
    match key {
        Node::Str(s) | Node::Real(s) | Node::Int(_, s) => s.clone(),
        Node::Bool(b) => b.to_string(),
        Node::Null => "null".to_owned(),
        _ => "?".to_owned(),
    }
}

fn visit_array<'de, V: Visitor<'de>>(array: Vec<Node>, path: Path, visitor: V) -> Result<V::Value> {
    let len = array.len();
    let mut seq = SeqDeserializer {
        iter: array.into_iter(),
        index: 0,
        path: path.clone(),
    };
    let value = visitor
        .visit_seq(&mut seq)
        .map_err(|e: Error| e.with_path(&path))?;
    if seq.iter.len() == 0 {
        Ok(value)
    } else {
        Err(Error::invalid_length(len, &"fewer elements in sequence").with_path(&path))
    }
}

fn visit_hash<'de, V: Visitor<'de>>(
    hash: Vec<(Node, Node)>,
    path: Path,
    visitor: V,
) -> Result<V::Value> {
    let remaining = hash.len();
    let mut map = MapDeserializer {
        iter: hash.into_iter(),
        remaining,
        label: String::new(),
        value: None,
        path: path.clone(),
    };
    visitor
        .visit_map(&mut map)
        .map_err(|e: Error| e.with_path(&path))
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let Self { yaml, path } = self;
        match yaml {
            Node::Null => visitor.visit_unit().map_err(|e: Error| e.with_path(&path)),
            Node::Bool(b) => visitor.visit_bool(b).map_err(|e: Error| e.with_path(&path)),
            Node::Int(i, _) => visitor.visit_i64(i).map_err(|e: Error| e.with_path(&path)),
            Node::Real(s) => match parse_real(&s) {
                Some(f) => visitor.visit_f64(f).map_err(|e: Error| e.with_path(&path)),
                None => visitor
                    .visit_string(s)
                    .map_err(|e: Error| e.with_path(&path)),
            },
            Node::Str(s) => visitor
                .visit_string(s)
                .map_err(|e: Error| e.with_path(&path)),
            Node::Seq(a) => visit_array(a, path, visitor),
            Node::Map(h) => visit_hash(h, path, visitor),
            Node::Bad => Err(Error::bad_value(&path)),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let path = self.path.clone();
        match self.yaml {
            Node::Null => visitor.visit_none().map_err(|e: Error| e.with_path(&path)),
            _ => visitor
                .visit_some(self)
                .map_err(|e: Error| e.with_path(&path)),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let Self { yaml, path } = self;
        match yaml {
            Node::Str(variant) => visitor
                .visit_enum(EnumDeserializer {
                    variant,
                    value: None,
                    path: path.clone(),
                })
                .map_err(|e: Error| e.with_path(&path)),
            Node::Map(hash) => {
                if hash.len() != 1 {
                    return Err(Error::new(format!(
                        "expected a map with exactly one key for enum, found {} keys",
                        hash.len()
                    ))
                    .with_path(&path));
                }
                let (key, value) = hash.into_iter().next().expect("length checked above");
                let Node::Str(variant) = key else {
                    return Err(Error::new("expected a string enum variant name").with_path(&path));
                };
                let mut child_path = path.clone();
                child_path.push_key(variant.clone());
                visitor
                    .visit_enum(EnumDeserializer {
                        variant,
                        value: Some(value),
                        path: child_path,
                    })
                    .map_err(|e: Error| e.with_path(&path))
            }
            Node::Bad => Err(Error::bad_value(&path)),
            _ => Err(Error::new("expected a string or single-key map for enum").with_path(&path)),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    /// Plain numbers and booleans are accepted where a string is expected
    /// (`port: 8080` into a `String`, `200:` as a map key), like serde_yaml.
    /// Integers keep the text they were written as (`0o17` reads as `"0o17"`).
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let Self { yaml, path } = self;
        let text = match yaml {
            Node::Int(_, s) | Node::Real(s) => s,
            Node::Bool(b) => b.to_string(),
            other => return Self { yaml: other, path }.deserialize_any(visitor),
        };
        visitor
            .visit_string(text)
            .map_err(|e: Error| e.with_path(&path))
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
    }
}

struct SeqDeserializer {
    iter: std::vec::IntoIter<Node>,
    index: usize,
    path: Path,
}

impl<'de> SeqAccess<'de> for &mut SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        let Some(yaml) = self.iter.next() else {
            return Ok(None);
        };
        let mut path = self.path.clone();
        path.push_index(self.index);
        self.index += 1;
        seed.deserialize(Deserializer::child(yaml, path)).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct MapDeserializer {
    iter: std::vec::IntoIter<(Node, Node)>,
    remaining: usize,
    label: String,
    value: Option<Node>,
    path: Path,
}

impl<'de> MapAccess<'de> for &mut MapDeserializer {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        let Some((key, value)) = self.iter.next() else {
            return Ok(None);
        };
        self.remaining -= 1;
        self.label = key_label(&key);
        self.value = Some(value);
        seed.deserialize(Deserializer::child(key, self.path.clone()))
            .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let value = self
            .value
            .take()
            .ok_or_else(|| Error::new("map value requested before key"))?;
        let mut path = self.path.clone();
        path.push_key(std::mem::take(&mut self.label));
        seed.deserialize(Deserializer::child(value, path))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<Node>,
    path: Path,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let variant = seed.deserialize(self.variant.into_deserializer())?;
        Ok((
            variant,
            VariantDeserializer {
                value: self.value,
                path: self.path,
            },
        ))
    }
}

struct VariantDeserializer {
    value: Option<Node>,
    path: Path,
}

impl VariantDeserializer {
    fn payload(self, expected: &'static str) -> Result<Deserializer> {
        match self.value {
            Some(yaml) => Ok(Deserializer::child(yaml, self.path)),
            None => Err(
                Error::new(format!("expected {expected} but found a unit variant",))
                    .with_path(&self.path),
            ),
        }
    }
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            None | Some(Node::Null) => Ok(()),
            Some(_) => {
                Err(Error::new("expected a unit variant but found a value").with_path(&self.path))
            }
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(self.payload("a newtype variant")?)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        de::Deserializer::deserialize_seq(self.payload("a tuple variant")?, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        de::Deserializer::deserialize_map(self.payload("a struct variant")?, visitor)
    }
}
