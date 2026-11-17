use serde_yaml_rust2::{from_str, Mapping, Value};

// ── pointer ──

#[test]
fn pointer_nested_mapping() {
    let v: Value = from_str("foo:\n  bar: 42").unwrap();
    assert_eq!(v.pointer("/foo/bar"), Some(&Value::from(42)));
}

#[test]
fn pointer_sequence_index() {
    let v: Value = from_str("items:\n  - name: first\n  - name: second").unwrap();
    assert_eq!(v.pointer("/items/0/name"), Some(&Value::from("first")));
    assert_eq!(v.pointer("/items/1/name"), Some(&Value::from("second")));
}

#[test]
fn pointer_missing_path_returns_none() {
    let v: Value = from_str("a: 1").unwrap();
    assert_eq!(v.pointer("/b"), None);
    assert_eq!(v.pointer("/a/deeper"), None);
}

#[test]
fn pointer_empty_string_returns_root() {
    let v: Value = from_str("x: 1").unwrap();
    assert_eq!(v.pointer(""), Some(&v));
}

#[test]
fn pointer_no_leading_slash_returns_none() {
    let v: Value = from_str("a: 1").unwrap();
    assert_eq!(v.pointer("a"), None);
}

#[test]
fn pointer_rfc6901_escaping() {
    let yaml = "\"a/b\": 1\n\"~x\": 2";
    let v: Value = from_str(yaml).unwrap();
    // ~1 decodes to /
    assert_eq!(v.pointer("/a~1b"), Some(&Value::from(1)));
    // ~0 decodes to ~
    assert_eq!(v.pointer("/~0x"), Some(&Value::from(2)));
}

#[test]
fn pointer_sequence_out_of_bounds() {
    let v: Value = from_str("list:\n  - a\n  - b").unwrap();
    assert_eq!(v.pointer("/list/5"), None);
}

// ── pointer_mut ──

#[test]
fn pointer_mut_modify_nested() {
    let mut v: Value = from_str("a:\n  b: 1").unwrap();
    *v.pointer_mut("/a/b").unwrap() = Value::from(99);
    assert_eq!(v.pointer("/a/b"), Some(&Value::from(99)));
}

#[test]
fn pointer_mut_missing_returns_none() {
    let mut v: Value = from_str("a: 1").unwrap();
    assert!(v.pointer_mut("/missing").is_none());
}

// ── Mapping::entry ──

#[test]
fn mapping_entry_insert_or_update() {
    let mut m = Mapping::new();

    m.entry(Value::from("count")).or_insert(Value::from(0));
    assert_eq!(m.get(&Value::from("count")), Some(&Value::from(0)));

    m.entry(Value::from("count")).and_modify(|v| {
        if let Some(n) = v.as_i64() {
            *v = Value::from(n + 1);
        }
    });
    assert_eq!(m.get(&Value::from("count")), Some(&Value::from(1)));
}

// ── Mapping::merge ──

#[test]
fn mapping_merge_overwrites_and_adds() {
    let mut base = Mapping::new();
    base.insert(Value::from("a"), Value::from(1));
    base.insert(Value::from("b"), Value::from(2));

    let mut patch = Mapping::new();
    patch.insert(Value::from("b"), Value::from(99));
    patch.insert(Value::from("c"), Value::from(3));

    base.merge(patch);

    assert_eq!(base.get(&Value::from("a")), Some(&Value::from(1)));
    assert_eq!(base.get(&Value::from("b")), Some(&Value::from(99)));
    assert_eq!(base.get(&Value::from("c")), Some(&Value::from(3)));
    assert_eq!(base.len(), 3);
}

#[test]
fn mapping_merge_empty() {
    let mut base = Mapping::new();
    base.insert(Value::from("a"), Value::from(1));
    base.merge(Mapping::new());
    assert_eq!(base.len(), 1);
}

// ── Value::deep_merge ──

#[test]
fn deep_merge_nested_mappings() {
    let mut base: Value = from_str("server:\n  host: localhost\n  port: 8080\nname: app").unwrap();
    let patch: Value = from_str("server:\n  port: 9090\n  tls: true\nversion: 2").unwrap();

    base.deep_merge(patch);

    assert_eq!(base["server"]["host"], "localhost");
    assert_eq!(base["server"]["port"].as_i64(), Some(9090));
    assert_eq!(base["server"]["tls"].as_bool(), Some(true));
    assert_eq!(base["name"], "app");
    assert_eq!(base["version"].as_i64(), Some(2));
}

#[test]
fn deep_merge_sequences_extend() {
    let mut base: Value = from_str("tags:\n  - a\n  - b").unwrap();
    let patch: Value = from_str("tags:\n  - c").unwrap();

    base.deep_merge(patch);

    let tags = base["tags"].as_sequence().unwrap();
    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0], "a");
    assert_eq!(tags[1], "b");
    assert_eq!(tags[2], "c");
}

#[test]
fn deep_merge_mapping_vs_non_mapping_replaces() {
    let mut base: Value = from_str("x:\n  a: 1").unwrap();
    let patch: Value = from_str("x: overwritten").unwrap();

    base.deep_merge(patch);

    assert_eq!(base["x"], "overwritten");
}

#[test]
fn deep_merge_non_mapping_vs_mapping_replaces() {
    let mut base: Value = from_str("x: simple").unwrap();
    let patch: Value = from_str("x:\n  nested: true").unwrap();

    base.deep_merge(patch);

    assert_eq!(base["x"]["nested"].as_bool(), Some(true));
}

#[test]
fn deep_merge_scalar_replaces() {
    let mut base = Value::from(1);
    base.deep_merge(Value::from(2));
    assert_eq!(base.as_i64(), Some(2));
}
