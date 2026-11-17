use serde_json::json;
use serde_yaml_rust2::{from_str, to_string, to_string_with_options, SerializeOptions};

fn round_trip(v: &serde_json::Value) -> String {
    let yaml = to_string(v).unwrap();
    let back: serde_json::Value =
        from_str(&yaml).unwrap_or_else(|e| panic!("cannot read back {yaml:?}: {e}"));
    assert_eq!(&back, v, "round trip changed the value; yaml:\n{yaml}");
    yaml
}

#[test]
fn safe_strings_are_plain() {
    let yaml = round_trip(&json!({
        "url": "http://localhost:8787",
        "dot": ".",
        "path": "/users/{id}",
        "jsonpath": "$.items[0].name",
        "regex": "/^application\\/json/",
        "dash": "-x",
        "apostrophe": "it's",
        "unicode": "日本語",
    }));
    assert_eq!(
        yaml,
        "url: http://localhost:8787\ndot: .\npath: /users/{id}\njsonpath: $.items[0].name\n\
         regex: /^application\\/json/\ndash: -x\napostrophe: it's\nunicode: 日本語\n"
    );
}

#[test]
fn strings_that_would_change_meaning_are_quoted() {
    let cases = [
        ("true", "'true'"),
        ("null", "'null'"),
        ("~", "'~'"),
        ("123", "'123'"),
        ("1.5", "'1.5'"),
        (".inf", "'.inf'"),
        ("0x1F", "'0x1F'"),
        ("yes", "'yes'"),
        ("", "''"),
        (" x", "' x'"),
        ("x ", "'x '"),
        ("a: b", "'a: b'"),
        ("a #b", "'a #b'"),
        ("x:", "'x:'"),
        ("-", "'-'"),
        ("- x", "'- x'"),
        ("*x", "'*x'"),
        ("&x", "'&x'"),
        ("!x", "'!x'"),
        ("#x", "'#x'"),
        ("{{ env.TOKEN }}", "'{{ env.TOKEN }}'"),
        ("[a]", "'[a]'"),
        ("---", "'---'"),
        ("...", "'...'"),
        ("it's: x", "'it''s: x'"),
    ];
    for (s, expected) in cases {
        let yaml = round_trip(&json!({ "k": s }));
        assert_eq!(yaml, format!("k: {expected}\n"), "for {s:?}");
    }
}

#[test]
fn control_characters_use_double_quotes() {
    for s in [
        "a\tb",
        "a\rb",
        "nul\0",
        "bell\x07",
        "bom\u{FEFF}",
        "ls\u{2028}",
        "nel\u{85}",
    ] {
        let yaml = round_trip(&json!({ "k": s }));
        assert!(yaml.starts_with("k: \""), "{s:?} -> {yaml}");
    }
}

#[test]
fn null_is_written_as_null() {
    assert_eq!(round_trip(&json!({ "q": null })), "q: null\n");
}

#[test]
fn multiline_strings_use_literal_blocks() {
    assert_eq!(
        round_trip(&json!({ "d": "line one\nline two\n" })),
        "d: |\n  line one\n  line two\n"
    );
    assert_eq!(round_trip(&json!({ "d": "a\nb" })), "d: |-\n  a\n  b\n");
    assert_eq!(round_trip(&json!({ "d": "a\n\n" })), "d: |+\n  a\n\n");
    assert_eq!(
        round_trip(&json!({ "d": "a\n\nb\n" })),
        "d: |\n  a\n\n  b\n"
    );
    assert_eq!(
        round_trip(&json!({ "l": ["x\ny"] })),
        "l:\n  - |-\n    x\n    y\n"
    );
}

#[test]
fn multiline_strings_that_cannot_be_literal_are_quoted() {
    for s in [" lead\nx", "tab\t\nx", "cr\r\nx", "\n\n", "\n"] {
        let yaml = round_trip(&json!({ "k": s }));
        assert!(yaml.starts_with("k: \""), "{s:?} -> {yaml}");
    }
}

#[test]
fn nested_collections() {
    let yaml = round_trip(&json!({
        "tags": ["a", "b"],
        "list": [{ "a": 1, "b": [1, 2] }, { "c": {} }],
        "matrix": [[1, 2], [3]],
        "empty_list": [],
        "empty_map": {},
    }));
    assert_eq!(
        yaml,
        "tags:\n  - a\n  - b\nlist:\n  - a: 1\n    b:\n      - 1\n      - 2\n  - c: {}\n\
         matrix:\n  - - 1\n    - 2\n  - - 3\nempty_list: []\nempty_map: {}\n"
    );
}

#[test]
fn indent_option_applies_to_nested_nodes() {
    let v = json!({ "a": { "b": [{ "c": 1, "d": { "e": 2 } }] } });
    let yaml = to_string_with_options(&v, &SerializeOptions::new().indent(4)).unwrap();
    assert_eq!(
        yaml,
        "a:\n    b:\n        - c: 1\n          d:\n              e: 2\n"
    );
    let back: serde_json::Value = from_str(&yaml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn flow_options_write_scalar_collections_inline() {
    let v = json!({ "ids": [1, 2, 3], "point": { "x": 1, "label": "a, b" }, "nested": [[1]] });
    let opts = SerializeOptions::new()
        .flow_sequences(true)
        .flow_mappings(true);
    let yaml = to_string_with_options(&v, &opts).unwrap();
    assert_eq!(
        yaml,
        "ids: [1, 2, 3]\npoint: {x: 1, label: 'a, b'}\nnested:\n  - [1]\n"
    );
    let back: serde_json::Value = from_str(&yaml).unwrap();
    assert_eq!(back, v);
}

#[test]
fn top_level_scalars() {
    assert_eq!(to_string("http://x").unwrap(), "http://x\n");
    assert_eq!(to_string("true").unwrap(), "'true'\n");
    assert_eq!(to_string(&5).unwrap(), "5\n");
    assert_eq!(to_string(&()).unwrap(), "null\n");
    assert_eq!(to_string("a\nb").unwrap(), "|-\n  a\n  b\n");
}
