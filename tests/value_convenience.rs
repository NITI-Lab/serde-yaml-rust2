use serde_yaml_rust2::{Mapping, Number, Value};

// ── From impls ──

#[test]
fn value_from_bool() {
    assert_eq!(Value::from(true), Value::Bool(true));
    assert_eq!(Value::from(false), Value::Bool(false));
}

#[test]
fn value_from_string() {
    let v = Value::from(String::from("hello"));
    assert_eq!(v, Value::String("hello".into()));
}

#[test]
fn value_from_str_ref() {
    let v = Value::from("world");
    assert_eq!(v, Value::String("world".into()));
}

#[test]
fn value_from_i64() {
    let v = Value::from(42i64);
    assert_eq!(v, Value::Number(Number::from(42i64)));
}

#[test]
fn value_from_i32() {
    let v = Value::from(-7i32);
    assert_eq!(v, Value::Number(Number::from(-7i32)));
}

#[test]
fn value_from_u64() {
    let v = Value::from(100u64);
    assert_eq!(v, Value::Number(Number::from(100u64)));
}

#[test]
fn value_from_f64() {
    let v = Value::from(3.14f64);
    assert_eq!(v, Value::Number(Number::from(3.14f64)));
}

#[test]
fn value_from_number() {
    let n = Number::from(99i64);
    let v = Value::from(n.clone());
    assert_eq!(v, Value::Number(n));
}

#[test]
fn value_from_sequence() {
    let seq = vec![Value::from(1), Value::from(2)];
    let v = Value::from(seq.clone());
    assert_eq!(v, Value::Sequence(seq));
}

#[test]
fn value_from_mapping() {
    let mut m = Mapping::new();
    m.insert(Value::from("key"), Value::from("val"));
    let v = Value::from(m.clone());
    assert_eq!(v, Value::Mapping(m));
}

#[test]
fn value_from_small_integers() {
    let _ = Value::from(1i8);
    let _ = Value::from(2i16);
    let _ = Value::from(3isize);
    let _ = Value::from(4u8);
    let _ = Value::from(5u16);
    let _ = Value::from(6u32);
    let _ = Value::from(7usize);
    let _ = Value::from(8f32);
}

#[test]
fn value_from_iterator() {
    let v: Value = vec![1, 2, 3].into_iter().collect();
    assert!(v.is_sequence());
    assert_eq!(v[0], Value::Number(Number::from(1i64)));
}

// ── Accessor methods ──

#[test]
fn as_bool_returns_some_for_bool() {
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::Bool(false).as_bool(), Some(false));
}

#[test]
fn as_bool_returns_none_for_non_bool() {
    assert_eq!(Value::from(42).as_bool(), None);
    assert_eq!(Value::from("true").as_bool(), None);
    assert_eq!(Value::Null.as_bool(), None);
}

#[test]
fn as_str_returns_some_for_string() {
    assert_eq!(Value::from("hello").as_str(), Some("hello"));
}

#[test]
fn as_str_returns_none_for_non_string() {
    assert_eq!(Value::from(42).as_str(), None);
    assert_eq!(Value::Null.as_str(), None);
}

#[test]
fn as_i64_returns_some_for_integer() {
    assert_eq!(Value::from(42i64).as_i64(), Some(42));
    assert_eq!(Value::from(-10i64).as_i64(), Some(-10));
}

#[test]
fn as_i64_returns_none_for_float() {
    assert_eq!(Value::from(3.14f64).as_i64(), None);
}

#[test]
fn as_u64_returns_some_for_positive() {
    assert_eq!(Value::from(42u64).as_u64(), Some(42));
}

#[test]
fn as_u64_returns_none_for_negative() {
    assert_eq!(Value::from(-1i64).as_u64(), None);
}

#[test]
fn as_f64_returns_some_for_float() {
    let v = Value::from(2.5f64);
    assert!((v.as_f64().unwrap() - 2.5).abs() < f64::EPSILON);
}

#[test]
fn as_f64_returns_none_for_string() {
    assert_eq!(Value::from("2.5").as_f64(), None);
}

#[test]
fn as_sequence_returns_some() {
    let v: Value = vec![1, 2].into_iter().collect();
    assert!(v.as_sequence().is_some());
    assert_eq!(v.as_sequence().unwrap().len(), 2);
}

#[test]
fn as_sequence_returns_none_for_non_seq() {
    assert!(Value::from(42).as_sequence().is_none());
}

#[test]
fn as_sequence_mut_works() {
    let mut v: Value = vec![1, 2].into_iter().collect();
    v.as_sequence_mut().unwrap().push(Value::from(3));
    assert_eq!(v.as_sequence().unwrap().len(), 3);
}

#[test]
fn as_mapping_returns_some() {
    let mut m = Mapping::new();
    m.insert(Value::from("k"), Value::from("v"));
    let v = Value::from(m);
    assert!(v.as_mapping().is_some());
}

#[test]
fn as_mapping_returns_none_for_non_map() {
    assert!(Value::from("hello").as_mapping().is_none());
}

#[test]
fn as_mapping_mut_works() {
    let mut m = Mapping::new();
    m.insert(Value::from("k"), Value::from("v"));
    let mut v = Value::from(m);
    v.as_mapping_mut()
        .unwrap()
        .insert(Value::from("k2"), Value::from("v2"));
    assert_eq!(v.as_mapping().unwrap().len(), 2);
}

// ── is_* checks ──

#[test]
fn is_null() {
    assert!(Value::Null.is_null());
    assert!(!Value::from(0).is_null());
}

#[test]
fn is_bool() {
    assert!(Value::Bool(true).is_bool());
    assert!(!Value::from("true").is_bool());
}

#[test]
fn is_number() {
    assert!(Value::from(42).is_number());
    assert!(!Value::from("42").is_number());
}

#[test]
fn is_string() {
    assert!(Value::from("hi").is_string());
    assert!(!Value::from(42).is_string());
}

#[test]
fn is_sequence() {
    let v: Value = vec![1].into_iter().collect();
    assert!(v.is_sequence());
    assert!(!Value::from("list").is_sequence());
}

#[test]
fn is_mapping() {
    let mut m = Mapping::new();
    m.insert(Value::from("k"), Value::from("v"));
    assert!(Value::from(m).is_mapping());
    assert!(!Value::from(42).is_mapping());
}

// ── PartialEq with primitives ──

#[test]
fn partial_eq_str_ref() {
    let v = Value::from("hello");
    assert_eq!(v, "hello");
    assert_ne!(v, "world");
}

#[test]
fn partial_eq_str_ref_non_string_value() {
    assert_ne!(Value::from(42), "42");
    assert_ne!(Value::Null, "null");
    assert_ne!(Value::Bool(true), "true");
}

#[test]
fn partial_eq_bool() {
    assert_eq!(Value::Bool(true), true);
    assert_eq!(Value::Bool(false), false);
    assert_ne!(Value::Bool(true), false);
}

#[test]
fn partial_eq_bool_non_bool_value() {
    assert_ne!(Value::from(1), true);
    assert_ne!(Value::from("true"), true);
}

#[test]
fn partial_eq_i64() {
    assert_eq!(Value::from(42i64), 42i64);
    assert_eq!(Value::from(-1i64), -1i64);
    assert_ne!(Value::from(42i64), 43i64);
}

#[test]
fn partial_eq_i64_non_number_value() {
    assert_ne!(Value::from("42"), 42i64);
    assert_ne!(Value::Null, 0i64);
}

#[test]
fn partial_eq_u64() {
    assert_eq!(Value::from(100u64), 100u64);
    assert_ne!(Value::from(100u64), 101u64);
}

#[test]
fn partial_eq_u64_negative_value() {
    assert_ne!(Value::from(-1i64), 0u64);
}

#[test]
fn partial_eq_f64() {
    assert_eq!(Value::from(3.14f64), 3.14f64);
    assert_ne!(Value::from(3.14f64), 3.15f64);
}

#[test]
fn partial_eq_f64_nan() {
    let nan_val = Value::from(f64::NAN);
    assert_eq!(nan_val, f64::NAN);
}

#[test]
fn partial_eq_f64_zero_signs() {
    let pos_zero = Value::from(0.0f64);
    let neg_zero = Value::from(-0.0f64);
    assert_eq!(pos_zero, 0.0f64);
    assert_eq!(neg_zero, -0.0f64);
    assert_ne!(pos_zero, -0.0f64);
}

#[test]
fn partial_eq_f64_infinity() {
    assert_eq!(Value::from(f64::INFINITY), f64::INFINITY);
    assert_eq!(Value::from(f64::NEG_INFINITY), f64::NEG_INFINITY);
    assert_ne!(Value::from(f64::INFINITY), f64::NEG_INFINITY);
}

#[test]
fn partial_eq_f64_non_number_value() {
    assert_ne!(Value::from("3.14"), 3.14f64);
}

// ── Index operations ──

#[test]
fn index_by_str() {
    let yaml = "key: value";
    let v: Value = serde_yaml_rust2::from_str(yaml).unwrap();
    assert_eq!(v["key"], Value::from("value"));
}

#[test]
fn index_by_usize() {
    let v: Value = vec![10, 20, 30].into_iter().collect();
    assert_eq!(v[1], Value::from(20));
}

#[test]
fn index_missing_returns_null() {
    let v: Value = serde_yaml_rust2::from_str("x: 1").unwrap();
    assert!(v["missing"].is_null());
}
