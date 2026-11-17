# serde-yaml-rust2

[yaml-rust2](https://github.com/Ethiraric/yaml-rust2) based [serde](https://serde.rs/) integration for YAML.

A safe, pure-Rust YAML 1.2 serialization and deserialization library.

[![CI](https://github.com/NITI-Lab/serde-yaml-rust2/actions/workflows/ci.yml/badge.svg)](https://github.com/NITI-Lab/serde-yaml-rust2/actions/workflows/ci.yml)

## Why this crate?

[`serde_yaml`](https://crates.io/crates/serde_yaml) (the original, based on `unsafe-libyaml`) was archived by its maintainer.
This crate provides a drop-in serde integration built on **yaml-rust2**, which is:

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

### Compatibility with serde_yaml

- Plain numbers and booleans are accepted where a string is expected, so `port: 8080` reads into a `String` and `200:` reads as a string map key (e.g. OpenAPI `responses`).
- An empty input (or one with only comments) reads as `null`.
- Duplicate keys in a mapping are an error, as the YAML spec requires.

### Types

- **`Value`** — An enum representing any YAML value (`Null`, `Bool`, `Number`, `String`, `Sequence`, `Mapping`). Supports indexing with strings and integers.
- **`Mapping`** — An ordered map (`IndexMap<Value, Value>`) that preserves insertion order.
- **`Number`** — A YAML number: unsigned integer (`u64`), signed integer (`i64`), or float (`f64`). Handles `.nan`, `.inf`, and `-.inf`.
- **`Sequence`** — Type alias for `Vec<Value>`.

## Minimum Supported Rust Version

The MSRV is **1.85** (due to transitive dependency requirements).

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
