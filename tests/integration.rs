use serde::{Deserialize, Serialize};
use serde_yaml_rust2::{from_str, from_value, to_string, to_value, Mapping, Number, Value};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    name: String,
    debug: bool,
    count: u32,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Nested {
    outer: String,
    inner: Inner,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Inner {
    value: i64,
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Animal {
    Cat,
    Dog,
    Fish { color: String, count: u32 },
}

// ── round-trip tests ──

#[test]
fn round_trip_struct() {
    let cfg = Config {
        name: "test".into(),
        debug: true,
        count: 42,
        tags: vec!["a".into(), "b".into()],
    };
    let yaml = to_string(&cfg).unwrap();
    let back: Config = from_str(&yaml).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn round_trip_nested() {
    let n = Nested {
        outer: "hello".into(),
        inner: Inner {
            value: -7,
            label: Some("world".into()),
        },
    };
    let yaml = to_string(&n).unwrap();
    let back: Nested = from_str(&yaml).unwrap();
    assert_eq!(n, back);
}

#[test]
fn round_trip_option_none() {
    let n = Nested {
        outer: "x".into(),
        inner: Inner {
            value: 0,
            label: None,
        },
    };
    let yaml = to_string(&n).unwrap();
    let back: Nested = from_str(&yaml).unwrap();
    assert_eq!(n, back);
}

#[test]
fn round_trip_enum_unit() {
    let a = Animal::Cat;
    let yaml = to_string(&a).unwrap();
    let back: Animal = from_str(&yaml).unwrap();
    assert_eq!(a, back);
}

#[test]
fn round_trip_enum_struct() {
    let a = Animal::Fish {
        color: "gold".into(),
        count: 3,
    };
    let yaml = to_string(&a).unwrap();
    let back: Animal = from_str(&yaml).unwrap();
    assert_eq!(a, back);
}

// ── deserialization tests ──

#[test]
fn deserialize_basic_yaml() {
    let yaml = "
name: myapp
debug: false
count: 100
tags:
  - prod
  - stable
";
    let cfg: Config = from_str(yaml).unwrap();
    assert_eq!(cfg.name, "myapp");
    assert!(!cfg.debug);
    assert_eq!(cfg.count, 100);
    assert_eq!(cfg.tags, vec!["prod", "stable"]);
}

#[test]
fn deserialize_empty_string() {
    let result: Result<Config, _> = from_str("");
    assert!(result.is_err());
}

#[test]
fn deserialize_null_as_none() {
    let yaml = "
outer: test
inner:
  value: 5
  label: ~
";
    let n: Nested = from_str(yaml).unwrap();
    assert_eq!(n.inner.label, None);
}

// ── serialization tests ──

#[test]
fn serialize_produces_valid_yaml() {
    let cfg = Config {
        name: "app".into(),
        debug: false,
        count: 0,
        tags: vec![],
    };
    let yaml = to_string(&cfg).unwrap();
    assert!(yaml.contains("name:"));
    assert!(yaml.contains("debug:"));
    assert!(yaml.ends_with('\n'));
}

#[test]
fn serialize_no_document_marker() {
    let v = "hello";
    let yaml = to_string(&v).unwrap();
    assert!(!yaml.starts_with("---"));
}

// ── Value type tests ──

#[test]
fn value_from_str_round_trip() {
    let yaml = "
key: value
num: 42
flag: true
list:
  - 1
  - 2
";
    let v: Value = from_str(yaml).unwrap();
    assert_eq!(v["key"], Value::String("value".into()));
    assert_eq!(v["num"], Value::Number(Number::from(42i64)));
    assert_eq!(v["flag"], Value::Bool(true));
    assert!(v["list"].is_sequence());
}

#[test]
fn value_index_missing_returns_null() {
    let v: Value = from_str("x: 1").unwrap();
    assert!(v["missing"].is_null());
}

#[test]
fn value_to_value_from_value() {
    let cfg = Config {
        name: "test".into(),
        debug: true,
        count: 7,
        tags: vec!["x".into()],
    };
    let v = to_value(&cfg).unwrap();
    let back: Config = from_value(v).unwrap();
    assert_eq!(cfg, back);
}

// ── Number tests ──

#[test]
fn number_integer_types() {
    let n = Number::from(42i64);
    assert!(n.is_i64());
    assert!(n.is_u64());
    assert!(!n.is_f64());
    assert_eq!(n.as_i64(), Some(42));
    assert_eq!(n.as_u64(), Some(42));
}

#[test]
fn number_negative() {
    let n = Number::from(-5i64);
    assert!(n.is_i64());
    assert!(!n.is_u64());
    assert_eq!(n.as_i64(), Some(-5));
    assert_eq!(n.as_u64(), None);
}

#[test]
fn number_float() {
    let n = Number::from(2.5f64);
    assert!(n.is_f64());
    assert!(!n.is_i64());
    assert!(!n.is_u64());
    assert!((n.as_f64().unwrap() - 2.5).abs() < f64::EPSILON);
}

#[test]
fn number_special_floats() {
    let yaml = ".nan";
    let v: Value = from_str(yaml).unwrap();
    if let Value::Number(n) = v {
        assert!(n.is_nan());
    } else {
        panic!("expected Number, got {:?}", v);
    }
}

#[test]
fn number_infinity() {
    let yaml = ".inf";
    let v: Value = from_str(yaml).unwrap();
    if let Value::Number(n) = v {
        assert!(n.is_infinite());
        assert_eq!(n.as_f64(), Some(f64::INFINITY));
    } else {
        panic!("expected Number, got {:?}", v);
    }
}

#[test]
fn number_neg_infinity() {
    let yaml = "-.inf";
    let v: Value = from_str(yaml).unwrap();
    if let Value::Number(n) = v {
        assert!(n.is_infinite());
        assert_eq!(n.as_f64(), Some(f64::NEG_INFINITY));
    } else {
        panic!("expected Number, got {:?}", v);
    }
}

// ── Mapping tests ──

#[test]
fn mapping_basic_ops() {
    let mut m = Mapping::new();
    assert!(m.is_empty());
    m.insert(Value::String("a".into()), Value::from(1));
    m.insert(Value::String("b".into()), Value::from(2));
    assert_eq!(m.len(), 2);
    assert!(m.contains_key(&Value::String("a".into())));
    assert_eq!(
        m.get(&Value::String("a".into())),
        Some(&Value::Number(Number::from(1i64)))
    );
}

#[test]
fn mapping_preserves_order() {
    let yaml = "
z: 1
a: 2
m: 3
";
    let v: Value = from_str(yaml).unwrap();
    if let Value::Mapping(m) = v {
        let keys: Vec<_> = m.keys().collect();
        assert_eq!(keys[0], &Value::String("z".into()));
        assert_eq!(keys[1], &Value::String("a".into()));
        assert_eq!(keys[2], &Value::String("m".into()));
    } else {
        panic!("expected Mapping");
    }
}

// ── from_slice / from_reader ──

#[test]
fn from_slice_works() {
    let bytes = b"hello: world";
    let v: Value = serde_yaml_rust2::from_slice(bytes).unwrap();
    assert_eq!(v["hello"], Value::String("world".into()));
}

#[test]
fn from_reader_works() {
    let cursor = std::io::Cursor::new(b"n: 42");
    let v: Value = serde_yaml_rust2::from_reader(cursor).unwrap();
    assert_eq!(v["n"], Value::Number(Number::from(42i64)));
}

// ── to_writer ──

#[test]
fn to_writer_works() {
    let mut buf = Vec::new();
    serde_yaml_rust2::to_writer(&mut buf, &"test").unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("test"));
}

// ── edge cases ──

#[test]
fn large_integer() {
    let yaml = "9223372036854775807"; // i64::MAX
    let v: Value = from_str(yaml).unwrap();
    assert_eq!(v.as_i64(), Some(i64::MAX));
}

#[test]
fn unicode_strings() {
    let yaml = "msg: こんにちは世界";
    let v: Value = from_str(yaml).unwrap();
    assert_eq!(v["msg"], Value::String("こんにちは世界".into()));
}

#[test]
fn multiline_string() {
    let yaml = "
text: |
  line one
  line two
  line three
";
    let v: Value = from_str(yaml).unwrap();
    let text = v["text"].as_str().unwrap();
    assert!(text.contains("line one"));
    assert!(text.contains("line three"));
}

#[test]
fn nested_sequences() {
    let yaml = "
matrix:
  - [1, 2, 3]
  - [4, 5, 6]
";
    let v: Value = from_str(yaml).unwrap();
    assert!(v["matrix"].is_sequence());
    assert!(v["matrix"][0].is_sequence());
}

#[test]
fn boolean_values() {
    let yaml = "
a: true
b: false
";
    let v: Value = from_str(yaml).unwrap();
    assert_eq!(v["a"], Value::Bool(true));
    assert_eq!(v["b"], Value::Bool(false));
}

#[test]
fn null_variants() {
    for input in &["~", "null"] {
        let v: Value = from_str(input).unwrap();
        assert!(v.is_null(), "expected null for input: {}", input);
    }
}

#[test]
fn error_has_location_for_invalid_yaml() {
    let yaml = "a: [unterminated";
    let err = serde_yaml_rust2::from_str::<Value>(yaml).unwrap_err();
    let loc = err.location().expect("should have location");
    assert!(loc.line() >= 1);
    assert!(loc.column() >= 1);
}

#[test]
fn error_display_includes_location() {
    let yaml = "a: {unterminated";
    let err = serde_yaml_rust2::from_str::<Value>(yaml).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("line"),
        "error should contain location info: {}",
        msg
    );
}

#[test]
fn multi_document_deserialize() {
    let yaml = "---\nname: first\n---\nname: second\n---\nname: third\n";
    let docs: Vec<std::collections::HashMap<String, String>> =
        serde_yaml_rust2::from_str_multidoc(yaml).unwrap();
    assert_eq!(docs.len(), 3);
    assert_eq!(docs[0]["name"], "first");
    assert_eq!(docs[1]["name"], "second");
    assert_eq!(docs[2]["name"], "third");
}

#[test]
fn multi_document_single() {
    let yaml = "hello: world\n";
    let docs: Vec<Value> = serde_yaml_rust2::from_str_multidoc(yaml).unwrap();
    assert_eq!(docs.len(), 1);
}

#[test]
fn multi_document_empty() {
    let yaml = "";
    let docs: Vec<Value> = serde_yaml_rust2::from_str_multidoc(yaml).unwrap();
    assert_eq!(docs.len(), 0);
}

#[test]
fn multi_document_typed() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Item {
        id: u32,
    }

    let yaml = "---\nid: 1\n---\nid: 2\n";
    let items: Vec<Item> = serde_yaml_rust2::from_str_multidoc(yaml).unwrap();
    assert_eq!(items, vec![Item { id: 1 }, Item { id: 2 }]);
}
