/// Property-based round-trip tests for serde-yaml-rust2.
///
/// Uses [`proptest`] to generate random inputs and verify that
/// serialization followed by deserialization produces the original value.
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use serde_yaml_rust2::{from_str, to_string, Mapping, Number, Value};
use std::collections::HashMap;

/// Strings safe as YAML scalars: start with a letter, contain only
/// alphanumeric characters, underscores, and spaces.
fn safe_string() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_ ]{0,30}"
}

/// Strings safe as mapping keys (no spaces for cleaner YAML).
fn safe_key() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]{0,15}"
}

// ── Primitive round-trips ──

proptest! {
    #[test]
    fn bool_round_trip(val in any::<bool>()) {
        let yaml = to_string(&val).unwrap();
        let back: bool = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }

    #[test]
    fn i64_round_trip(val in any::<i64>()) {
        let yaml = to_string(&val).unwrap();
        let back: i64 = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }

    #[test]
    fn u64_round_trip(val in 0..=i64::MAX as u64) {
        let yaml = to_string(&val).unwrap();
        let back: u64 = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }

    #[test]
    fn f64_round_trip(val in any::<f64>().prop_filter("not NaN", |v| !v.is_nan())) {
        let yaml = to_string(&val).unwrap();
        let back: f64 = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }

    #[test]
    fn string_round_trip(val in safe_string()) {
        let yaml = to_string(&val).unwrap();
        let back: String = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }
}

// ── Struct round-trip ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct FlatStruct {
    name: String,
    count: i64,
    flag: bool,
    score: f64,
}

prop_compose! {
    fn arb_flat_struct()(
        name in safe_string(),
        count in any::<i64>(),
        flag in any::<bool>(),
        score in any::<f64>().prop_filter("finite", |v| v.is_finite()),
    ) -> FlatStruct {
        FlatStruct { name, count, flag, score }
    }
}

proptest! {
    #[test]
    fn flat_struct_round_trip(val in arb_flat_struct()) {
        let yaml = to_string(&val).unwrap();
        let back: FlatStruct = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }
}

// ── Vec round-trips ──

proptest! {
    #[test]
    fn vec_i64_round_trip(val in prop::collection::vec(any::<i64>(), 0..20)) {
        let yaml = to_string(&val).unwrap();
        let back: Vec<i64> = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }

    #[test]
    fn vec_string_round_trip(val in prop::collection::vec(safe_string(), 0..20)) {
        let yaml = to_string(&val).unwrap();
        let back: Vec<String> = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }
}

// ── Nested struct round-trip ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Outer {
    label: String,
    inner: InnerStruct,
    items: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct InnerStruct {
    x: i64,
    y: i64,
}

prop_compose! {
    fn arb_inner()(x in any::<i64>(), y in any::<i64>()) -> InnerStruct {
        InnerStruct { x, y }
    }
}

prop_compose! {
    fn arb_outer()(
        label in safe_string(),
        inner in arb_inner(),
        items in prop::collection::vec(any::<i64>(), 0..10),
    ) -> Outer {
        Outer { label, inner, items }
    }
}

proptest! {
    #[test]
    fn nested_struct_round_trip(val in arb_outer()) {
        let yaml = to_string(&val).unwrap();
        let back: Outer = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }
}

// ── Option round-trip ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WithOption {
    required: i64,
    optional: Option<i64>,
}

proptest! {
    #[test]
    fn option_round_trip(
        required in any::<i64>(),
        optional in prop::option::of(any::<i64>()),
    ) {
        let val = WithOption { required, optional };
        let yaml = to_string(&val).unwrap();
        let back: WithOption = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }
}

// ── HashMap round-trip ──

proptest! {
    #[test]
    fn hashmap_round_trip(
        val in prop::collection::hash_map(safe_key(), any::<i64>(), 0..20),
    ) {
        let yaml = to_string(&val).unwrap();
        let back: HashMap<String, i64> = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }
}

// ── Enum round-trip ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum TestEnum {
    Unit,
    Newtype(i64),
    Struct { a: String, b: bool },
}

fn arb_test_enum() -> impl Strategy<Value = TestEnum> {
    prop_oneof![
        Just(TestEnum::Unit),
        any::<i64>().prop_map(TestEnum::Newtype),
        (safe_string(), any::<bool>()).prop_map(|(a, b)| TestEnum::Struct { a, b }),
    ]
}

proptest! {
    #[test]
    fn enum_round_trip(val in arb_test_enum()) {
        let yaml = to_string(&val).unwrap();
        let back: TestEnum = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }
}

// ── Value round-trip ──

fn arb_number() -> impl Strategy<Value = Number> {
    prop_oneof![
        any::<i64>().prop_map(Number::from),
        any::<f64>()
            .prop_filter("not NaN", |v| !v.is_nan())
            .prop_filter("finite", |v| v.is_finite())
            .prop_map(Number::from),
    ]
}

fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        arb_number().prop_map(Value::Number),
        safe_string().prop_map(Value::String),
    ];
    leaf.prop_recursive(
        3,  // depth
        64, // max nodes
        10, // items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..5).prop_map(Value::Sequence),
                prop::collection::vec((safe_key().prop_map(Value::String), inner), 0..5).prop_map(
                    |pairs| {
                        let mut m = Mapping::new();
                        for (k, v) in pairs {
                            m.insert(k, v);
                        }
                        Value::Mapping(m)
                    },
                ),
            ]
        },
    )
}

proptest! {
    #[test]
    fn value_round_trip(val in arb_value()) {
        let yaml = to_string(&val).unwrap();
        let back: Value = from_str(&yaml).unwrap();
        prop_assert_eq!(val, back);
    }
}
