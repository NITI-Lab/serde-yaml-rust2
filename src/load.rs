//! Loads YAML text into [`Node`] trees.
//!
//! This follows `yaml_rust2::YamlLoader` (same scalar resolution, anchors and
//! duplicate-key errors), but keeps the text of plain integers as written, so
//! that `c: 0o17` read into a `String` gives `"0o17"` rather than `"15"`.

use std::collections::{BTreeMap, HashSet};

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser, Tag};
use yaml_rust2::scanner::{Marker, ScanError, TScalarStyle};
use yaml_rust2::Yaml;

/// A loaded YAML node.
#[derive(Clone, Debug)]
pub(crate) enum Node {
    Null,
    Bool(bool),
    /// The value and the text it was written as (e.g. `0x1F`, `+5`).
    Int(i64, String),
    Real(String),
    Str(String),
    Seq(Vec<Node>),
    Map(Vec<(Node, Node)>),
    Bad,
}

impl Node {
    /// The resolved value, without the written text.
    fn to_yaml(&self) -> Yaml {
        match self {
            Node::Null => Yaml::Null,
            Node::Bool(b) => Yaml::Boolean(*b),
            Node::Int(i, _) => Yaml::Integer(*i),
            Node::Real(s) => Yaml::Real(s.clone()),
            Node::Str(s) => Yaml::String(s.clone()),
            Node::Seq(a) => Yaml::Array(a.iter().map(Node::to_yaml).collect()),
            Node::Map(m) => Yaml::Hash(m.iter().map(|(k, v)| (k.to_yaml(), v.to_yaml())).collect()),
            Node::Bad => Yaml::BadValue,
        }
    }
}

impl From<Yaml> for Node {
    fn from(yaml: Yaml) -> Self {
        match yaml {
            Yaml::Null => Node::Null,
            Yaml::Boolean(b) => Node::Bool(b),
            Yaml::Integer(i) => Node::Int(i, i.to_string()),
            Yaml::Real(s) => Node::Real(s),
            Yaml::String(s) => Node::Str(s),
            Yaml::Array(a) => Node::Seq(a.into_iter().map(Node::from).collect()),
            Yaml::Hash(h) => Node::Map(
                h.into_iter()
                    .map(|(k, v)| (Node::from(k), Node::from(v)))
                    .collect(),
            ),
            Yaml::Alias(_) | Yaml::BadValue => Node::Bad,
        }
    }
}

/// A sequence or mapping that is still open.
enum Frame {
    Seq(Vec<Node>, usize),
    Map {
        entries: Vec<(Node, Node)>,
        keys: HashSet<Yaml>,
        key: Option<Node>,
        anchor: usize,
    },
}

#[derive(Default)]
struct Loader {
    docs: Vec<Node>,
    root: Option<Node>,
    stack: Vec<Frame>,
    anchors: BTreeMap<usize, Node>,
    error: Option<ScanError>,
}

/// Load every document in `source`.
pub(crate) fn load_from_str(source: &str) -> Result<Vec<Node>, ScanError> {
    let mut loader = Loader::default();
    Parser::new_from_str(source).load(&mut loader, true)?;
    match loader.error {
        Some(e) => Err(e),
        None => Ok(loader.docs),
    }
}

impl MarkedEventReceiver for Loader {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        if self.error.is_some() {
            return;
        }
        if let Err(e) = self.handle(ev, mark) {
            self.error = Some(e);
        }
    }
}

impl Loader {
    fn handle(&mut self, ev: Event, mark: Marker) -> Result<(), ScanError> {
        match ev {
            Event::DocumentEnd => {
                let doc = self.root.take().unwrap_or(Node::Bad);
                self.docs.push(doc);
            }
            Event::SequenceStart(anchor, _) => self.stack.push(Frame::Seq(Vec::new(), anchor)),
            Event::MappingStart(anchor, _) => self.stack.push(Frame::Map {
                entries: Vec::new(),
                keys: HashSet::new(),
                key: None,
                anchor,
            }),
            Event::SequenceEnd | Event::MappingEnd => {
                let (node, anchor) = match self.stack.pop() {
                    Some(Frame::Seq(items, anchor)) => (Node::Seq(items), anchor),
                    Some(Frame::Map {
                        entries, anchor, ..
                    }) => (Node::Map(entries), anchor),
                    None => return Ok(()),
                };
                self.insert(node, anchor, mark)?;
            }
            Event::Scalar(text, style, anchor, tag) => {
                let node = resolve_scalar(text, style, tag.as_ref());
                self.insert(node, anchor, mark)?;
            }
            Event::Alias(id) => {
                let node = self.anchors.get(&id).cloned().unwrap_or(Node::Bad);
                self.insert(node, 0, mark)?;
            }
            Event::DocumentStart | Event::Nothing | Event::StreamStart | Event::StreamEnd => {}
        }
        Ok(())
    }

    fn insert(&mut self, node: Node, anchor: usize, mark: Marker) -> Result<(), ScanError> {
        // Valid anchor ids start at 1.
        if anchor > 0 {
            self.anchors.insert(anchor, node.clone());
        }
        match self.stack.last_mut() {
            None => self.root = Some(node),
            Some(Frame::Seq(items, _)) => items.push(node),
            Some(Frame::Map {
                entries, keys, key, ..
            }) => match key.take() {
                None => *key = Some(node),
                Some(k) => {
                    let resolved = k.to_yaml();
                    if !keys.insert(resolved.clone()) {
                        return Err(ScanError::new_string(
                            mark,
                            format!("{resolved:?}: duplicated key in mapping"),
                        ));
                    }
                    entries.push((k, node));
                }
            },
        }
        Ok(())
    }
}

/// Resolve a scalar the same way `yaml_rust2::YamlLoader` does, keeping the
/// written text of plain integers.
fn resolve_scalar(text: String, style: TScalarStyle, tag: Option<&Tag>) -> Node {
    if style != TScalarStyle::Plain {
        return Node::Str(text);
    }
    if let Some(Tag { handle, suffix }) = tag {
        if handle != "tag:yaml.org,2002:" {
            return Node::Str(text);
        }
        return match suffix.as_str() {
            "bool" => match text.as_str() {
                "true" | "True" | "TRUE" => Node::Bool(true),
                "false" | "False" | "FALSE" => Node::Bool(false),
                _ => Node::Bad,
            },
            "int" => match text.parse::<i64>() {
                Ok(i) => Node::Int(i, text),
                Err(_) => Node::Bad,
            },
            "float" => match Yaml::from_str(&text) {
                Yaml::Real(_) => Node::Real(text),
                Yaml::Integer(_) if text.parse::<f64>().is_ok() => Node::Real(text),
                _ => Node::Bad,
            },
            "null" => match text.as_str() {
                "~" | "null" => Node::Null,
                _ => Node::Bad,
            },
            _ => Node::Str(text),
        };
    }
    match Yaml::from_str(&text) {
        Yaml::Integer(i) => Node::Int(i, text),
        other => Node::from(other),
    }
}
