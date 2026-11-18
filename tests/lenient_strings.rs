use serde::Deserialize;
use serde_yaml_rust2::{from_str, Value};
use std::collections::BTreeMap;

#[test]
fn scalars_read_as_strings() {
    let m: BTreeMap<String, String> = from_str("count: 1\nflag: true\nver: 1.50\n").unwrap();
    assert_eq!(m["count"], "1");
    assert_eq!(m["flag"], "true");
    assert_eq!(m["ver"], "1.50");
}

#[test]
fn numeric_keys_read_as_string_keys() {
    let v: serde_json::Value =
        from_str("responses:\n  200: {description: ok}\n  404: x\n").unwrap();
    assert_eq!(v["responses"]["200"]["description"], "ok");
    assert_eq!(v["responses"]["404"], "x");
}

#[test]
fn numeric_keys_still_read_as_numbers() {
    let m: BTreeMap<u16, String> = from_str("200: ok\n404: missing\n").unwrap();
    assert_eq!(m[&200], "ok");
    let v: Value = from_str("200: ok").unwrap();
    assert!(v.as_mapping().unwrap().keys().next().unwrap().is_number());
}

#[test]
fn null_is_not_a_string() {
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    struct S {
        name: String,
    }
    assert!(from_str::<S>("name: ~").is_err());
    let o: BTreeMap<String, Option<String>> = from_str("name: ~").unwrap();
    assert_eq!(o["name"], None);
}

#[test]
fn empty_input_reads_as_null() {
    let v: Value = from_str("").unwrap();
    assert!(v.is_null());
    let o: Option<BTreeMap<String, i64>> = from_str("# only a comment\n").unwrap();
    assert!(o.is_none());
}

#[test]
fn error_location_points_at_the_offending_line() {
    let err = from_str::<Value>("a: 1\nb: 2\na: 3\n").unwrap_err();
    let loc = err.location().expect("location");
    assert_eq!(loc.line(), 3);
    assert!(err.to_string().starts_with("line 3 "), "{}", err);
}

#[test]
fn integers_read_as_strings_keep_their_text() {
    let m: BTreeMap<String, String> =
        from_str("a: 0o17\nb: 0x1F\nc: +5\nd: 007\ne: -0\nf: 15\n").unwrap();
    assert_eq!(m["a"], "0o17");
    assert_eq!(m["b"], "0x1F");
    assert_eq!(m["c"], "+5");
    assert_eq!(m["d"], "007");
    assert_eq!(m["e"], "-0");
    assert_eq!(m["f"], "15");

    let keys: BTreeMap<String, i64> = from_str("0x10: 1\n").unwrap();
    assert_eq!(keys["0x10"], 1);
}

#[test]
fn non_decimal_integers_still_read_as_numbers() {
    let m: BTreeMap<String, i64> = from_str("a: 0o17\nb: 0x1F\nc: +5\n").unwrap();
    assert_eq!(m["a"], 15);
    assert_eq!(m["b"], 31);
    assert_eq!(m["c"], 5);
    let v: Value = from_str("a: 0o17").unwrap();
    assert_eq!(v["a"], Value::Number(15.into()));
}

#[test]
fn aliases_keep_integer_text() {
    let m: BTreeMap<String, String> = from_str("a: &x 0o17\nb: *x\n").unwrap();
    assert_eq!(m["b"], "0o17");
}

#[test]
fn equal_integer_keys_written_differently_are_duplicates() {
    let err = from_str::<Value>("15: a\n0o17: b\n").unwrap_err();
    assert!(err.to_string().contains("duplicated key"), "{}", err);
}

#[test]
fn deserializer_from_yaml_node_still_works() {
    use serde::Deserialize as _;
    let yaml = yaml_rust2::YamlLoader::load_from_str("n: 0o17")
        .unwrap()
        .remove(0);
    let m =
        BTreeMap::<String, String>::deserialize(serde_yaml_rust2::Deserializer::new(yaml)).unwrap();
    assert_eq!(m["n"], "15");
}
