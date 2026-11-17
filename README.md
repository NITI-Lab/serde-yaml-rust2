# serde-yaml-rust2

[yaml-rust2](https://github.com/Ethiraric/yaml-rust2) based [serde](https://serde.rs/) integration for YAML.

A safe, pure-Rust YAML 1.2 serialization and deserialization library.

[![CI](https://github.com/NITI-Lab/serde-yaml-rust2/actions/workflows/ci.yml/badge.svg)](https://github.com/NITI-Lab/serde-yaml-rust2/actions/workflows/ci.yml)

## Why this crate?

[`serde_yaml`](https://crates.io/crates/serde_yaml) (the original, based on `unsafe-libyaml`) was archived by its maintainer.
This crate provides a serde integration with a `serde_yaml`-like API, built on **yaml-rust2**, which is:

- **Pure Rust** — no C dependencies, no `unsafe` code (this crate is `#![forbid(unsafe_code)]`)
- **YAML 1.2** compliant via yaml-rust2's Core Schema
- **Actively maintained**

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde-yaml-rust2 = "0.5"
```

### Serialize and Deserialize structs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    debug: bool,
    count: u32,
    tags: Vec<String>,
}

fn main() -> serde_yaml_rust2::Result<()> {
    // Deserialize from YAML
    let yaml = "
name: my-app
debug: true
count: 42
tags:
  - web
  - rust
";
    let config: Config = serde_yaml_rust2::from_str(yaml)?;
    println!("{:?}", config);

    // Serialize to YAML
    let output = serde_yaml_rust2::to_string(&config)?;
    println!("{}", output);

    Ok(())
}
```

### Work with dynamic values

```rust
use serde_yaml_rust2::Value;

fn main() -> serde_yaml_rust2::Result<()> {
    let yaml = "
server:
  host: localhost
  port: 8080
  features:
    - logging
    - metrics
";
    let v: Value = serde_yaml_rust2::from_str(yaml)?;

    // Index into the value
    assert_eq!(v["server"]["host"], Value::String("localhost".into()));
    assert_eq!(v["server"]["port"], Value::Number(8080.into()));

    // Missing keys return Value::Null
    assert!(v["server"]["missing"].is_null());

    Ok(())
}
```

### Convert between Value and typed data

```rust
use serde::{Deserialize, Serialize};
use serde_yaml_rust2::{from_value, to_value, Value};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Point {
    x: f64,
    y: f64,
}

fn main() -> serde_yaml_rust2::Result<()> {
    let point = Point { x: 1.0, y: 2.0 };

    // Struct → Value
    let value: Value = to_value(&point)?;

    // Value → Struct
    let restored: Point = from_value(value)?;
    assert_eq!(point, restored);

    Ok(())
}
```

## API

| Function | Description |
|---|---|
| `from_str(s)` | Deserialize from a YAML string |
| `from_slice(v)` | Deserialize from a UTF-8 byte slice |
| `from_reader(rdr)` | Deserialize from any `Read` |
| `from_str_multidoc(s)` | Deserialize every `---`-separated document into a `Vec` |
| `to_string(value)` | Serialize to a YAML string |
| `to_writer(writer, value)` | Serialize to any `Write` |
| `to_string_with_options(value, opts)` | Serialize with `SerializeOptions` (indent, flow style, `---` marker) |
| `from_value(value)` | Convert a `Value` to a typed value |
| `to_value(value)` | Convert a typed value to a `Value` |

### Output format

`to_string` writes block-style YAML meant to be read and edited by people:

- Strings are written plain when that reads back as the same string (`url: http://localhost:8080`, `path: /users/{id}`), single-quoted when they would otherwise mean something else (`'true'`, `'123'`, `'a: b'`, and YAML 1.1 booleans such as `'yes'`), and double-quoted with escapes when they contain control characters.
- Multi-line strings are written as `|` literal blocks.
- `None` / unit is written as `null`.
- Nested sequences are indented under their key (`tags:\n  - a`).

### Types

- **`Value`** — An enum representing any YAML value (`Null`, `Bool`, `Number`, `String`, `Sequence`, `Mapping`). Supports indexing with strings and integers.
- **`Mapping`** — An ordered map (`IndexMap<Value, Value>`) that preserves insertion order.
- **`Number`** — A YAML number: unsigned integer (`u64`), signed integer (`i64`), or float (`f64`). Handles `.nan`, `.inf`, and `-.inf`.
- **`Sequence`** — Type alias for `Vec<Value>`.

## Migrating from serde_yaml

The common entry points (`from_str`, `from_slice`, `from_reader`, `to_string`, `to_writer`, `from_value`, `to_value`, `Value`, `Mapping`, `Number`, `Error`) have the same names and shapes as in `serde_yaml`, so most projects only need to change their dependency.

### 1. Swap the dependency

The least invasive way is to keep the `serde_yaml` name in your code and rename the package in `Cargo.toml`:

```toml
[dependencies]
# serde_yaml = "0.9"
serde_yaml = { package = "serde-yaml-rust2", version = "0.5" }
```

Or switch the name everywhere:

```toml
[dependencies]
serde-yaml-rust2 = "0.5"
```

```rust
// before
use serde_yaml::{from_str, to_string, Value};
// after
use serde_yaml_rust2::{from_str, to_string, Value};
```

### 2. Check the API differences

| serde_yaml | serde-yaml-rust2 | Notes |
|---|---|---|
| `from_str`, `from_slice`, `from_reader` | same | Deserialize into owned types only (`DeserializeOwned`); fields such as `&'a str` must become `String` or `Cow<'_, str>`. |
| `to_string`, `to_writer` | same | See [Output format](#output-format). `to_string_with_options` / `to_writer_with_options` add indent and flow-style control. |
| `from_value`, `to_value` | same | |
| `Deserializer::from_str(s)` iterated over documents | `from_str_multidoc::<T>(s)` or `Deserializer::from_str_documents(s)` | Returns a `Vec` instead of an iterator. |
| `Serializer::new(writer)` | `to_writer` / `to_writer_with_options` | `Serializer` is a unit struct that builds a YAML tree; it does not take a writer. |
| `Value::Tagged`, `TaggedValue`, `Tag` | — | Not supported (see below). |
| `value.apply_merge()` | — | `<<` merge keys are not applied (see below). `Value::deep_merge` merges two values but is not the same feature. |
| `mapping.get("key")` | `mapping.get(&Value::from("key"))` or `value["key"]` / `value.get("key")` | `Mapping` methods take `&Value` keys. |
| `serde_yaml::with::singleton_map` | not needed | Enums are already written as single-key maps (see below). |
| `Error::location()` → `Option<Location>` | `Error::location()` → `Option<&Location>` | `Location` has `line()` and `column()` (1-indexed) but no `index()`. `Error::path()` adds the path in the document, e.g. `.servers[0].port`. |

### 3. Check the behavior differences

**Enums are written as maps, not YAML tags.** `serde_yaml` 0.9 writes enum variants with tags (`!New 1`); this crate writes them the way `serde_yaml` 0.8 and `serde_json` do, and reads only that form:

```yaml
# serde_yaml 0.9
- Unit
- !New 1
- !Str
  a: 3
```

```yaml
# serde-yaml-rust2
- Unit
- New: 1
- Str:
    a: 3
```

YAML that you wrote with `serde_yaml` 0.9 and that contains tagged enum variants must be converted before it can be read. Unit variants (`- Unit`) are the same in both.

**Custom tags are dropped.** `!foo bar` reads as the string `"bar"`; the tag is not kept anywhere.

**Merge keys (`<<`) are not applied.** Anchors and aliases (`&a` / `*a`) work, but `<<: *a` stays as a literal key named `<<`.

**Scalars follow YAML 1.2.** `yes`, `no`, `on`, `off` are strings, not booleans, and `1_000` is a string, not a number. When writing, such strings are quoted (`'yes'`) so YAML 1.1 parsers read them back as strings.

**Reading is lenient where `serde_yaml` users tend to expect it:**

- Plain numbers and booleans are accepted where a string is expected, so `port: 8080` reads into a `String` and `200:` reads as a string map key (e.g. OpenAPI `responses`).
- An empty input (or one with only comments) reads as `null`.
- Duplicate keys in a mapping are an error, as the YAML spec requires.

**Output text can differ.** Indentation, quoting, and multi-line strings follow the rules in [Output format](#output-format). Values read back the same, but if you compare YAML text in tests or snapshots, regenerate them.

## Minimum Supported Rust Version

The MSRV is **1.85** (due to transitive dependency requirements).

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
