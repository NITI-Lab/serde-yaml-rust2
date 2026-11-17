use serde::Deserialize;
use serde_yaml_rust2::Value;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Outer {
    inner: Inner,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Inner {
    value: u32,
}

#[test]
fn type_mismatch_in_nested_field_includes_path() {
    let yaml = "inner:\n  value: not_a_number";
    let err = serde_yaml_rust2::from_str::<Outer>(yaml).unwrap_err();
    let path = err.path().expect("error should have a path");
    assert!(
        path.contains("inner") && path.contains("value"),
        "path should include .inner.value, got: {}",
        path
    );
}

#[test]
fn missing_field_includes_path() {
    let yaml = "inner:\n  wrong_key: 42";
    let err = serde_yaml_rust2::from_str::<Outer>(yaml).unwrap_err();
    let path = err.path().expect("error should have a path");
    assert!(
        path.contains("inner"),
        "path should include .inner, got: {}",
        path
    );
}

#[test]
fn error_in_sequence_element_includes_index() {
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct List {
        items: Vec<Inner>,
    }

    let yaml = "items:\n  - value: 1\n  - value: not_a_number\n  - value: 3";
    let err = serde_yaml_rust2::from_str::<List>(yaml).unwrap_err();
    let path = err.path().expect("error should have a path");
    assert!(
        path.contains("[1]"),
        "path should include [1] for second element, got: {}",
        path
    );
}

#[test]
fn path_appears_in_display_output() {
    let yaml = "inner:\n  value: not_a_number";
    let err = serde_yaml_rust2::from_str::<Outer>(yaml).unwrap_err();
    let display = err.to_string();
    assert!(
        display.contains("at "),
        "Display should include 'at <path>', got: {}",
        display
    );
    assert!(
        display.contains(".inner"),
        "Display should include path, got: {}",
        display
    );
}

#[test]
fn root_error_has_path() {
    let yaml = "42";
    let err = serde_yaml_rust2::from_str::<Outer>(yaml).unwrap_err();
    let path = err.path().expect("root error should have a path");
    assert_eq!(path, ".", "root path should be '.'");
}

#[test]
fn value_deserialization_no_path() {
    let err = serde_yaml_rust2::from_str::<Value>("a: [unterminated").unwrap_err();
    // Parse errors (ScanError) have location but not necessarily a path
    assert!(err.location().is_some());
}
