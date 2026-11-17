use crate::value::Value;
use indexmap::map::{Entry, IndexMap};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, Serializer};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, Default, Debug)]
pub struct Mapping {
    map: IndexMap<Value, Value>,
}

impl Mapping {
    pub fn new() -> Self {
        Mapping::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Mapping {
            map: IndexMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn insert(&mut self, k: Value, v: Value) -> Option<Value> {
        self.map.insert(k, v)
    }

    pub fn remove(&mut self, k: &Value) -> Option<Value> {
        self.map.shift_remove(k)
    }

    pub fn get(&self, k: &Value) -> Option<&Value> {
        self.map.get(k)
    }

    pub fn get_mut(&mut self, k: &Value) -> Option<&mut Value> {
        self.map.get_mut(k)
    }

    pub fn contains_key(&self, k: &Value) -> bool {
        self.map.contains_key(k)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, Value, Value> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> indexmap::map::IterMut<'_, Value, Value> {
        self.map.iter_mut()
    }

    pub fn keys(&self) -> indexmap::map::Keys<'_, Value, Value> {
        self.map.keys()
    }

    pub fn values(&self) -> indexmap::map::Values<'_, Value, Value> {
        self.map.values()
    }

    pub fn entry(&mut self, k: Value) -> Entry<'_, Value, Value> {
        self.map.entry(k)
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

impl PartialEq for Mapping {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl Eq for Mapping {}

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

impl IntoIterator for Mapping {
    type Item = (Value, Value);
    type IntoIter = indexmap::map::IntoIter<Value, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a> IntoIterator for &'a Mapping {
    type Item = (&'a Value, &'a Value);
    type IntoIter = indexmap::map::Iter<'a, Value, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a> IntoIterator for &'a mut Mapping {
    type Item = (&'a Value, &'a mut Value);
    type IntoIter = indexmap::map::IterMut<'a, Value, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

impl FromIterator<(Value, Value)> for Mapping {
    fn from_iter<I: IntoIterator<Item = (Value, Value)>>(iter: I) -> Self {
        Mapping {
            map: IndexMap::from_iter(iter),
        }
    }
}

impl Extend<(Value, Value)> for Mapping {
    fn extend<I: IntoIterator<Item = (Value, Value)>>(&mut self, iter: I) {
        self.map.extend(iter);
    }
}

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

impl<'de> Deserialize<'de> for Mapping {
    fn deserialize<D>(deserializer: D) -> Result<Mapping, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MappingVisitor)
    }
}
