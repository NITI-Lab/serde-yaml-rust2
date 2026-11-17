use serde::{Deserialize, Serialize};
use serde_yaml_rust2::{from_str, to_string, Value};

// 1. #[serde(rename = "...")] on a field
#[test]
fn test_rename_field() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Item {
        #[serde(rename = "item_name")]
        name: String,
        count: u32,
    }

    let yaml = "item_name: widget\ncount: 5\n";
    let item: Item = from_str(yaml).unwrap();
    assert_eq!(item.name, "widget");
    assert_eq!(item.count, 5);

    let serialized = to_string(&item).unwrap();
    assert!(serialized.contains("item_name:"));
    assert!(!serialized.contains("name:"));
    let back: Item = from_str(&serialized).unwrap();
    assert_eq!(item, back);
}

// 2. #[serde(rename_all = "camelCase")] on a struct
#[test]
fn test_rename_all_camel_case() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    struct Settings {
        max_retries: u32,
        base_url: String,
        is_enabled: bool,
    }

    let yaml = "maxRetries: 3\nbaseUrl: http://example.com\nisEnabled: true\n";
    let settings: Settings = from_str(yaml).unwrap();
    assert_eq!(settings.max_retries, 3);
    assert_eq!(settings.base_url, "http://example.com");
    assert!(settings.is_enabled);

    let serialized = to_string(&settings).unwrap();
    assert!(serialized.contains("maxRetries:"));
    assert!(serialized.contains("baseUrl:"));
    assert!(serialized.contains("isEnabled:"));
    let back: Settings = from_str(&serialized).unwrap();
    assert_eq!(settings, back);
}

// 3. #[serde(rename_all = "SCREAMING_SNAKE_CASE")] on an enum
#[test]
fn test_rename_all_screaming_snake_case_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum LogLevel {
        Debug,
        Info,
        Warning,
        CriticalError,
    }

    let yaml = "WARNING\n";
    let level: LogLevel = from_str(yaml).unwrap();
    assert_eq!(level, LogLevel::Warning);

    let yaml2 = "CRITICAL_ERROR\n";
    let level2: LogLevel = from_str(yaml2).unwrap();
    assert_eq!(level2, LogLevel::CriticalError);

    let serialized = to_string(&LogLevel::Debug).unwrap();
    assert!(serialized.contains("DEBUG"));
    let back: LogLevel = from_str(&serialized).unwrap();
    assert_eq!(back, LogLevel::Debug);
}

// 4. #[serde(default)] on a field
#[test]
fn test_default_field() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Config {
        name: String,
        #[serde(default)]
        debug: bool,
        #[serde(default)]
        count: u32,
    }

    let yaml = "name: app\n";
    let config: Config = from_str(yaml).unwrap();
    assert_eq!(config.name, "app");
    assert!(!config.debug);
    assert_eq!(config.count, 0);
}

// 5. #[serde(default = "path")] with a custom default function
#[test]
fn test_default_custom_function() {
    fn default_port() -> u16 {
        8080
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Server {
        host: String,
        #[serde(default = "default_port")]
        port: u16,
    }

    let yaml = "host: localhost\n";
    let server: Server = from_str(yaml).unwrap();
    assert_eq!(server.host, "localhost");
    assert_eq!(server.port, 8080);

    let yaml_with_port = "host: localhost\nport: 3000\n";
    let server2: Server = from_str(yaml_with_port).unwrap();
    assert_eq!(server2.port, 3000);
}

// 6. #[serde(skip)] on a field
#[test]
fn test_skip_field() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Record {
        id: u32,
        #[serde(skip)]
        cached: bool,
    }

    let record = Record {
        id: 42,
        cached: true,
    };
    let serialized = to_string(&record).unwrap();
    assert!(serialized.contains("id:"));
    assert!(!serialized.contains("cached"));

    let back: Record = from_str(&serialized).unwrap();
    assert_eq!(back.id, 42);
    assert!(!back.cached); // skipped field gets Default
}

// 7. #[serde(skip_serializing_if = "Option::is_none")]
#[test]
fn test_skip_serializing_if_none() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Profile {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,
    }

    let with_email = Profile {
        name: "Alice".into(),
        email: Some("alice@example.com".into()),
    };
    let yaml_with = to_string(&with_email).unwrap();
    assert!(yaml_with.contains("email:"));

    let without_email = Profile {
        name: "Bob".into(),
        email: None,
    };
    let yaml_without = to_string(&without_email).unwrap();
    assert!(!yaml_without.contains("email"));
    assert!(yaml_without.contains("name:"));
}

// 8. #[serde(alias = "...")] — alias deserializes correctly
#[test]
fn test_alias() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Entry {
        #[serde(alias = "title")]
        name: String,
        value: i32,
    }

    let yaml_canonical = "name: hello\nvalue: 1\n";
    let e1: Entry = from_str(yaml_canonical).unwrap();
    assert_eq!(e1.name, "hello");

    let yaml_alias = "title: world\nvalue: 2\n";
    let e2: Entry = from_str(yaml_alias).unwrap();
    assert_eq!(e2.name, "world");
}

// 9. #[serde(flatten)] on a nested struct
#[test]
fn test_flatten() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Base {
        id: u32,
        name: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Extended {
        #[serde(flatten)]
        base: Base,
        extra: String,
    }

    let yaml = "id: 1\nname: thing\nextra: bonus\n";
    let ext: Extended = from_str(yaml).unwrap();
    assert_eq!(ext.base.id, 1);
    assert_eq!(ext.base.name, "thing");
    assert_eq!(ext.extra, "bonus");

    let serialized = to_string(&ext).unwrap();
    assert!(serialized.contains("id:"));
    assert!(serialized.contains("name:"));
    assert!(serialized.contains("extra:"));
    // Flattened fields should be at top level, not nested
    assert!(!serialized.contains("base:"));
}

// 10. #[serde(untagged)] enum
#[test]
fn test_untagged_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(untagged)]
    enum StringOrInt {
        Int(i64),
        Str(String),
    }

    let yaml_int = "42\n";
    let val: StringOrInt = from_str(yaml_int).unwrap();
    assert_eq!(val, StringOrInt::Int(42));

    let yaml_str = "hello\n";
    let val2: StringOrInt = from_str(yaml_str).unwrap();
    assert_eq!(val2, StringOrInt::Str("hello".into()));

    let roundtrip = to_string(&StringOrInt::Int(99)).unwrap();
    let back: StringOrInt = from_str(&roundtrip).unwrap();
    assert_eq!(back, StringOrInt::Int(99));
}

// 11. #[serde(tag = "type")] internally tagged enum
#[test]
fn test_internally_tagged_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type")]
    enum Shape {
        Circle { radius: f64 },
        Rectangle { width: f64, height: f64 },
    }

    let yaml = "type: Circle\nradius: 5.0\n";
    let shape: Shape = from_str(yaml).unwrap();
    assert_eq!(shape, Shape::Circle { radius: 5.0 });

    let yaml_rect = "type: Rectangle\nwidth: 10.0\nheight: 20.0\n";
    let rect: Shape = from_str(yaml_rect).unwrap();
    assert_eq!(
        rect,
        Shape::Rectangle {
            width: 10.0,
            height: 20.0,
        }
    );

    let serialized = to_string(&shape).unwrap();
    assert!(serialized.contains("type:"));
    assert!(serialized.contains("radius:"));
    let back: Shape = from_str(&serialized).unwrap();
    assert_eq!(back, Shape::Circle { radius: 5.0 });
}

// 12. #[serde(tag = "type", content = "data")] adjacently tagged enum
#[test]
fn test_adjacently_tagged_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type", content = "data")]
    enum Message {
        Text(String),
        Numbers(Vec<i32>),
    }

    let yaml = "type: Text\ndata: hello world\n";
    let msg: Message = from_str(yaml).unwrap();
    assert_eq!(msg, Message::Text("hello world".into()));

    let serialized = to_string(&msg).unwrap();
    assert!(serialized.contains("type:"));
    assert!(serialized.contains("data:"));
    let back: Message = from_str(&serialized).unwrap();
    assert_eq!(back, Message::Text("hello world".into()));
}

// 13. #[serde(deny_unknown_fields)] — extra keys produce an error
#[test]
fn test_deny_unknown_fields() {
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Strict {
        name: String,
        age: u32,
    }

    let yaml_ok = "name: Alice\nage: 30\n";
    let strict: Strict = from_str(yaml_ok).unwrap();
    assert_eq!(strict.name, "Alice");
    assert_eq!(strict.age, 30);

    let yaml_extra = "name: Bob\nage: 25\nunknown: bad\n";
    let result: Result<Strict, _> = from_str(yaml_extra);
    assert!(result.is_err(), "should reject unknown fields");
}

// 14. #[serde(transparent)] on a newtype-like struct
#[test]
fn test_transparent() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(transparent)]
    struct UserId {
        value: u64,
    }

    let yaml = "42\n";
    let id: UserId = from_str(yaml).unwrap();
    assert_eq!(id.value, 42);

    let serialized = to_string(&id).unwrap();
    let back: UserId = from_str(&serialized).unwrap();
    assert_eq!(back.value, 42);
    // Transparent means it serializes as the inner value, not a map
    let v: Value = from_str(&serialized).unwrap();
    assert!(v.is_number());
}

// 15. Tuple struct round-trip
#[test]
fn test_tuple_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Point(f64, f64, f64);

    let yaml = "- 1.0\n- 2.0\n- 3.0\n";
    let point: Point = from_str(yaml).unwrap();
    assert_eq!(point, Point(1.0, 2.0, 3.0));

    let serialized = to_string(&point).unwrap();
    let back: Point = from_str(&serialized).unwrap();
    assert_eq!(back, Point(1.0, 2.0, 3.0));
}
