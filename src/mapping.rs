use crate::value::Value;
use indexmap::map::{Entry, IndexMap};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, Serializer};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A YAML mapping from [`Value`] keys to [`Value`] values.
///
/// Insertion order is preserved: iteration yields entries in the order they
/// were inserted (or appeared in the document). Removing a key keeps the
/// remaining entries in order.
///
/// # Examples
///
/// ```rust
/// use serde_yaml_rust2::{Mapping, Value};
///
/// let mut m = Mapping::new();
/// m.insert(Value::from("b"), Value::from(1));
/// m.insert(Value::from("a"), Value::from(2));
/// let keys: Vec<_> = m.keys().collect();
/// assert_eq!(keys, [&Value::from("b"), &Value::from("a")]);
/// ```
#[derive(Clone, Default, Debug)]
pub struct Mapping {
    map: IndexMap<Value, Value>,
}

impl Mapping {
    /// Creates an empty mapping.
    pub fn new() -> Self {
        Mapping::default()
    }

    /// Creates an empty mapping with space for at least `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Mapping {
            map: IndexMap::with_capacity(capacity),
        }
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the mapping has no entries.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Inserts a key/value pair, returning the previous value if the key existed.
    ///
    /// An existing key keeps its original position; a new key is appended.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::{Mapping, Value};
    ///
    /// let mut m = Mapping::new();
    /// assert_eq!(m.insert(Value::from("k"), Value::from(1)), None);
    /// assert_eq!(m.insert(Value::from("k"), Value::from(2)), Some(Value::from(1)));
    /// ```
    pub fn insert(&mut self, k: Value, v: Value) -> Option<Value> {
        self.map.insert(k, v)
    }

    /// Removes a key, returning its value if present. The order of the remaining entries is preserved.
    pub fn remove(&mut self, k: &Value) -> Option<Value> {
        self.map.shift_remove(k)
    }

    /// Returns a reference to the value for `k`, if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::{Mapping, Value};
    ///
    /// let mut m = Mapping::new();
    /// m.insert(Value::from("k"), Value::from(true));
    /// assert_eq!(m.get(&Value::from("k")), Some(&Value::Bool(true)));
    /// ```
    pub fn get(&self, k: &Value) -> Option<&Value> {
        self.map.get(k)
    }

    /// Returns a mutable reference to the value for `k`, if present.
    pub fn get_mut(&mut self, k: &Value) -> Option<&mut Value> {
        self.map.get_mut(k)
    }

    /// Returns `true` if the mapping contains the key `k`.
    pub fn contains_key(&self, k: &Value) -> bool {
        self.map.contains_key(k)
    }

    /// Iterates over entries in insertion order.
    pub fn iter(&self) -> indexmap::map::Iter<'_, Value, Value> {
        self.map.iter()
    }

    /// Iterates over entries in insertion order, with mutable access to the values.
    pub fn iter_mut(&mut self) -> indexmap::map::IterMut<'_, Value, Value> {
        self.map.iter_mut()
    }

    /// Iterates over keys in insertion order.
    pub fn keys(&self) -> indexmap::map::Keys<'_, Value, Value> {
        self.map.keys()
    }

    /// Iterates over values in insertion order.
    pub fn values(&self) -> indexmap::map::Values<'_, Value, Value> {
        self.map.values()
    }

    /// Gets the given key's entry for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::{Mapping, Value};
    ///
    /// let mut m = Mapping::new();
    /// m.entry(Value::from("n")).or_insert(Value::from(0));
    /// assert_eq!(m.len(), 1);
    /// ```
    pub fn entry(&mut self, k: Value) -> Entry<'_, Value, Value> {
        self.map.entry(k)
    }

    /// Shallow-merges all entries from `other` into `self`.
    ///
    /// Keys already present in `self` are overwritten with `other`'s values.
    /// New keys are appended in the order they appear in `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::{Mapping, Value};
    ///
    /// let mut base = Mapping::new();
    /// base.insert(Value::from("a"), Value::from(1));
    /// base.insert(Value::from("b"), Value::from(2));
    ///
    /// let mut patch = Mapping::new();
    /// patch.insert(Value::from("b"), Value::from(99));
    /// patch.insert(Value::from("c"), Value::from(3));
    ///
    /// base.merge(patch);
    /// assert_eq!(base.get(&Value::from("a")), Some(&Value::from(1)));
    /// assert_eq!(base.get(&Value::from("b")), Some(&Value::from(99)));
    /// assert_eq!(base.get(&Value::from("c")), Some(&Value::from(3)));
    /// ```
    pub fn merge(&mut self, other: Mapping) {
        for (k, v) in other {
            self.map.insert(k, v);
        }
    }

    pub(crate) fn from_yaml_hash(hash: yaml_rust2::yaml::Hash) -> Self {
        let mut map = IndexMap::with_capacity(hash.len());
        for (k, v) in hash {
            map.insert(Value::from(k), Value::from(v));
        }
        Mapping { map }
    }

    pub(crate) fn into_yaml_hash(self) -> yaml_rust2::yaml::Hash {
        let mut hash = yaml_rust2::yaml::Hash::with_capacity(self.map.len());
        for (k, v) in self.map {
            hash.insert(k.into(), v.into());
        }
        hash
    }
}

/// Two mappings are equal if they contain the same entries; entry order is ignored.
impl PartialEq for Mapping {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

/// Mappings have total equality because [`Value`] does.
impl Eq for Mapping {}

/// Hashes the mapping independently of entry order.
impl Hash for Mapping {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut acc = 0u64;
        for (k, v) in &self.map {
            let mut h = DefaultHasher::new();
            k.hash(&mut h);
            v.hash(&mut h);
            acc = acc.wrapping_add(h.finish());
        }
        state.write_usize(self.map.len());
        state.write_u64(acc);
    }
}

/// Consumes the mapping, yielding owned `(key, value)` pairs in insertion order.
impl IntoIterator for Mapping {
    type Item = (Value, Value);
    type IntoIter = indexmap::map::IntoIter<Value, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

/// Iterates over `(&key, &value)` pairs in insertion order.
impl<'a> IntoIterator for &'a Mapping {
    type Item = (&'a Value, &'a Value);
    type IntoIter = indexmap::map::Iter<'a, Value, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

/// Iterates over `(&key, &mut value)` pairs in insertion order.
impl<'a> IntoIterator for &'a mut Mapping {
    type Item = (&'a Value, &'a mut Value);
    type IntoIter = indexmap::map::IterMut<'a, Value, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

/// Builds a mapping from `(key, value)` pairs; later duplicates overwrite earlier values.
impl FromIterator<(Value, Value)> for Mapping {
    fn from_iter<I: IntoIterator<Item = (Value, Value)>>(iter: I) -> Self {
        Mapping {
            map: IndexMap::from_iter(iter),
        }
    }
}

/// Extends the mapping with `(key, value)` pairs.
impl Extend<(Value, Value)> for Mapping {
    fn extend<I: IntoIterator<Item = (Value, Value)>>(&mut self, iter: I) {
        self.map.extend(iter);
    }
}

/// Serializes the mapping as a map, in insertion order.
impl Serialize for Mapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_map(self.iter())
    }
}

pub(crate) struct MappingVisitor;

impl<'de> Visitor<'de> for MappingVisitor {
    type Value = Mapping;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a YAML mapping")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Mapping, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut mapping = Mapping::with_capacity(access.size_hint().unwrap_or(0).min(4096));
        while let Some((k, v)) = access.next_entry::<Value, Value>()? {
            match mapping.entry(k) {
                Entry::Occupied(_) => return Err(de::Error::custom("duplicate entry in mapping")),
                Entry::Vacant(slot) => {
                    slot.insert(v);
                }
            }
        }
        Ok(mapping)
    }
}

/// Deserializes a mapping, rejecting duplicate keys.
impl<'de> Deserialize<'de> for Mapping {
    fn deserialize<D>(deserializer: D) -> Result<Mapping, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MappingVisitor)
    }
}
