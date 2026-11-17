use serde::Serialize;
use serde_yaml_rust2::{to_string, to_string_with_options, to_writer_with_options, SerializeOptions};

#[derive(Serialize)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
struct Config {
    name: String,
    debug: bool,
    count: u32,
    tags: Vec<String>,
}

fn sample_config() -> Config {
    Config {
        name: "app".into(),
        debug: true,
        count: 42,
        tags: vec!["a".into(), "b".into()],
    }
}

#[test]
fn default_options_match_to_string() {
    let cfg = sample_config();
    let default = to_string(&cfg).unwrap();
    let with_opts = to_string_with_options(&cfg, &SerializeOptions::default()).unwrap();
    assert_eq!(default, with_opts);
}

#[test]
fn document_start_true_includes_marker() {
    let opts = SerializeOptions::new().document_start(true);
    let yaml = to_string_with_options(&"hello", &opts).unwrap();
    assert!(
        yaml.starts_with("---"),
        "expected document start marker, got: {}",
        yaml
    );
}

#[test]
fn document_start_false_strips_marker() {
    let opts = SerializeOptions::new().document_start(false);
    let yaml = to_string_with_options(&"hello", &opts).unwrap();
    assert!(
        !yaml.starts_with("---"),
        "should not contain document start marker: {}",
        yaml
    );
}

#[test]
fn indent_4_doubles_leading_spaces() {
    let cfg = sample_config();
    let opts = SerializeOptions::new().indent(4);
    let yaml = to_string_with_options(&cfg, &opts).unwrap();
    for line in yaml.lines() {
        let spaces = line.len() - line.trim_start().len();
        if spaces > 0 {
            assert_eq!(
                spaces % 4,
                0,
                "line '{}' has {} spaces, not a multiple of 4",
                line,
                spaces
            );
        }
    }
}

#[test]
fn indent_default_is_two_spaces() {
    let cfg = sample_config();
    let yaml = to_string_with_options(&cfg, &SerializeOptions::new()).unwrap();
    let indented_lines: Vec<_> = yaml
        .lines()
        .filter(|l| {
            let spaces = l.len() - l.trim_start().len();
            spaces > 0
        })
        .collect();
    assert!(!indented_lines.is_empty(), "should have indented lines");
    for line in indented_lines {
        let spaces = line.len() - line.trim_start().len();
        assert_eq!(
            spaces % 2,
            0,
            "line '{}' has {} spaces, not a multiple of 2",
            line,
            spaces
        );
    }
}

#[test]
fn indent_1_compresses_output() {
    let cfg = sample_config();
    let default_yaml = to_string(&cfg).unwrap();
    let opts = SerializeOptions::new().indent(1);
    let compact_yaml = to_string_with_options(&cfg, &opts).unwrap();
    assert!(
        compact_yaml.len() <= default_yaml.len(),
        "1-space indent should not be longer than 2-space"
    );
}

#[test]
fn builder_chaining() {
    let opts = SerializeOptions::new()
        .indent(4)
        .document_start(true)
        .flow_sequences(true)
        .flow_mappings(true);
    let yaml = to_string_with_options(&sample_config(), &opts).unwrap();
    assert!(yaml.starts_with("---"));
}

#[test]
fn to_writer_with_options_works() {
    let opts = SerializeOptions::new().document_start(true);
    let mut buf = Vec::new();
    to_writer_with_options(&mut buf, &"test", &opts).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.starts_with("---"));
}

#[test]
fn document_start_with_struct() {
    let cfg = sample_config();
    let opts = SerializeOptions::new().document_start(true);
    let yaml = to_string_with_options(&cfg, &opts).unwrap();
    assert!(yaml.starts_with("---"));
    assert!(yaml.contains("name:"));
}

#[test]
fn flow_options_accepted_without_panic() {
    let opts = SerializeOptions::new()
        .flow_sequences(true)
        .flow_mappings(true);
    let yaml = to_string_with_options(&sample_config(), &opts).unwrap();
    assert!(yaml.contains("name:"));
}

#[test]
fn serialize_options_is_clone() {
    let opts = SerializeOptions::new().indent(4);
    let opts2 = opts.clone();
    let a = to_string_with_options(&"x", &opts).unwrap();
    let b = to_string_with_options(&"x", &opts2).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serialize_options_is_debug() {
    let opts = SerializeOptions::new().indent(4).document_start(true);
    let debug = format!("{:?}", opts);
    assert!(debug.contains("indent"));
    assert!(debug.contains("document_start"));
}

#[test]
fn output_ends_with_newline() {
    let opts = SerializeOptions::new().document_start(true).indent(4);
    let yaml = to_string_with_options(&sample_config(), &opts).unwrap();
    assert!(yaml.ends_with('\n'));
}
