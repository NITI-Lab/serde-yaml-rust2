//! Block-style YAML emitter for `yaml_rust2::Yaml` trees.
//!
//! Strings are written plain when that reads back as the same string,
//! single-quoted when they only need protection from YAML syntax, as `|`
//! literal blocks when they span several lines, and double-quoted (with
//! escapes) otherwise.

use yaml_rust2::yaml::Hash;
use yaml_rust2::{Yaml, YamlLoader};

use crate::error::{Error, Result};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Block,
    Key,
    Flow,
}

pub(crate) struct Emitter<'a> {
    out: &'a mut String,
    indent: usize,
    flow_sequences: bool,
    flow_mappings: bool,
}

impl<'a> Emitter<'a> {
    pub(crate) fn new(
        out: &'a mut String,
        indent: usize,
        flow_sequences: bool,
        flow_mappings: bool,
    ) -> Self {
        Self {
            out,
            indent: indent.max(1),
            flow_sequences,
            flow_mappings,
        }
    }

    pub(crate) fn document(&mut self, node: &Yaml) -> Result<()> {
        match node {
            Yaml::Hash(h) if self.is_block(node) => self.block_map(h, 0, false),
            Yaml::Array(a) if self.is_block(node) => self.block_seq(a, 0, false),
            _ => self.inline_line(node, self.indent),
        }
    }

    fn is_block(&self, node: &Yaml) -> bool {
        match node {
            Yaml::Array(a) => !a.is_empty() && !self.flow_seq(a),
            Yaml::Hash(h) => !h.is_empty() && !self.flow_map(h),
            _ => false,
        }
    }

    fn flow_seq(&self, a: &[Yaml]) -> bool {
        self.flow_sequences && a.iter().all(is_scalar)
    }

    fn flow_map(&self, h: &Hash) -> bool {
        self.flow_mappings && h.iter().all(|(k, v)| is_scalar(k) && is_scalar(v))
    }

    fn pad(&mut self, col: usize) {
        self.out.extend(std::iter::repeat_n(' ', col));
    }

    fn block_map(&mut self, h: &Hash, col: usize, first_inline: bool) -> Result<()> {
        for (i, (k, v)) in h.iter().enumerate() {
            if i > 0 || !first_inline {
                self.pad(col);
            }
            self.key(k)?;
            self.out.push(':');
            match v {
                Yaml::Hash(child) if self.is_block(v) => {
                    self.out.push('\n');
                    self.block_map(child, col + self.indent, false)?;
                }
                Yaml::Array(child) if self.is_block(v) => {
                    self.out.push('\n');
                    self.block_seq(child, col + self.indent, false)?;
                }
                _ => {
                    self.out.push(' ');
                    self.inline_line(v, col + self.indent)?;
                }
            }
        }
        Ok(())
    }

    fn block_seq(&mut self, a: &[Yaml], col: usize, first_inline: bool) -> Result<()> {
        for (i, item) in a.iter().enumerate() {
            if i > 0 || !first_inline {
                self.pad(col);
            }
            self.out.push_str("- ");
            match item {
                Yaml::Hash(child) if self.is_block(item) => self.block_map(child, col + 2, true)?,
                Yaml::Array(child) if self.is_block(item) => {
                    self.block_seq(child, col + 2, true)?
                }
                _ => self.inline_line(item, col + 2)?,
            }
        }
        Ok(())
    }

    /// Write a node that fits after `key: ` or `- `, then end the line.
    /// `content_col` is the indentation of a literal block's lines.
    fn inline_line(&mut self, node: &Yaml, content_col: usize) -> Result<()> {
        if let Yaml::String(s) = node {
            if literal_ok(s) {
                self.literal(s, content_col);
                return Ok(());
            }
        }
        self.inline(node, Ctx::Block)?;
        self.out.push('\n');
        Ok(())
    }

    fn key(&mut self, k: &Yaml) -> Result<()> {
        match k {
            Yaml::Array(_) | Yaml::Hash(_) => self.flow(k),
            _ => self.inline(k, Ctx::Key),
        }
    }

    fn inline(&mut self, node: &Yaml, ctx: Ctx) -> Result<()> {
        match node {
            Yaml::String(s) => self.string(s, ctx),
            Yaml::Integer(i) => self.out.push_str(itoa::Buffer::new().format(*i)),
            Yaml::Real(r) => self.out.push_str(r),
            Yaml::Boolean(b) => self.out.push_str(if *b { "true" } else { "false" }),
            Yaml::Null => self.out.push_str("null"),
            Yaml::Array(_) | Yaml::Hash(_) => return self.flow(node),
            Yaml::Alias(_) | Yaml::BadValue => {
                return Err(Error::new("cannot serialize an alias or a bad YAML value"))
            }
        }
        Ok(())
    }

    fn flow(&mut self, node: &Yaml) -> Result<()> {
        match node {
            Yaml::Array(a) => {
                self.out.push('[');
                for (i, item) in a.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.inline(item, Ctx::Flow)?;
                }
                self.out.push(']');
            }
            Yaml::Hash(h) => {
                self.out.push('{');
                for (i, (k, v)) in h.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.inline(k, Ctx::Flow)?;
                    self.out.push_str(": ");
                    self.inline(v, Ctx::Flow)?;
                }
                self.out.push('}');
            }
            other => self.inline(other, Ctx::Flow)?,
        }
        Ok(())
    }

    fn string(&mut self, s: &str, ctx: Ctx) {
        if plain_ok(s, ctx) {
            self.out.push_str(s);
        } else if s.chars().all(single_quotable) {
            self.out.push('\'');
            self.out.push_str(&s.replace('\'', "''"));
            self.out.push('\'');
        } else {
            self.double_quoted(s);
        }
    }

    fn double_quoted(&mut self, s: &str) {
        self.out.push('"');
        for c in s.chars() {
            match c {
                '"' => self.out.push_str("\\\""),
                '\\' => self.out.push_str("\\\\"),
                '\n' => self.out.push_str("\\n"),
                '\t' => self.out.push_str("\\t"),
                '\r' => self.out.push_str("\\r"),
                '\0' => self.out.push_str("\\0"),
                c if single_quotable(c) || c == ' ' => self.out.push(c),
                c if (c as u32) <= 0xFF => self.out.push_str(&format!("\\x{:02X}", c as u32)),
                c if (c as u32) <= 0xFFFF => self.out.push_str(&format!("\\u{:04X}", c as u32)),
                c => self.out.push_str(&format!("\\U{:08X}", c as u32)),
            }
        }
        self.out.push('"');
    }

    fn literal(&mut self, s: &str, content_col: usize) {
        let body = s.trim_end_matches('\n');
        let trailing = s.len() - body.len();
        self.out.push_str(match trailing {
            0 => "|-",
            1 => "|",
            _ => "|+",
        });
        self.out.push('\n');
        for line in body.split('\n') {
            if !line.is_empty() {
                self.pad(content_col);
                self.out.push_str(line);
            }
            self.out.push('\n');
        }
        for _ in 1..trailing {
            self.out.push('\n');
        }
    }
}

fn is_scalar(node: &Yaml) -> bool {
    match node {
        Yaml::Array(_) | Yaml::Hash(_) => false,
        Yaml::String(s) => !s.contains('\n'),
        _ => true,
    }
}

/// Characters that can appear unescaped inside single quotes (YAML
/// printable characters, minus line breaks, tabs and the byte order mark).
fn single_quotable(c: char) -> bool {
    matches!(c,
        '\x20'..='\x7E'
        | '\u{A0}'..='\u{2027}'
        | '\u{202A}'..='\u{D7FF}'
        | '\u{E000}'..='\u{FEFE}'
        | '\u{FF00}'..='\u{FFFD}'
        | '\u{10000}'..='\u{10FFFF}')
}

/// YAML 1.1 booleans: plain strings in YAML 1.2, but quoted so that older
/// parsers do not turn them into `true` / `false`.
const YAML11_BOOLS: &[&str] = &[
    "y", "Y", "yes", "Yes", "YES", "n", "N", "no", "No", "NO", "on", "On", "ON", "off", "Off",
    "OFF",
];

fn plain_ok(s: &str, ctx: Ctx) -> bool {
    let Some(first) = s.chars().next() else {
        return false;
    };
    if s.starts_with(' ') || s.ends_with(' ') || !s.chars().all(single_quotable) {
        return false;
    }
    if ctx == Ctx::Key && s.len() > 1024 {
        return false;
    }
    let flow_indicator = |c: char| matches!(c, ',' | '[' | ']' | '{' | '}');
    if ctx == Ctx::Flow && s.contains(|c: char| flow_indicator(c) || c == ':') {
        return false;
    }
    if matches!(
        first,
        ',' | '['
            | ']'
            | '{'
            | '}'
            | '#'
            | '&'
            | '*'
            | '!'
            | '|'
            | '>'
            | '\''
            | '"'
            | '%'
            | '@'
            | '`'
    ) {
        return false;
    }
    if matches!(first, '-' | '?' | ':') {
        match s[1..].chars().next() {
            None | Some(' ') => return false,
            Some(c) if ctx == Ctx::Flow && flow_indicator(c) => return false,
            Some(_) => {}
        }
    }
    if s.contains(": ") || s.contains(" #") || s.ends_with(':') {
        return false;
    }
    if YAML11_BOOLS.contains(&s) {
        return false;
    }
    // Anything that does not read back as this exact string (numbers,
    // booleans, null, document markers, ...) needs quotes.
    matches!(
        YamlLoader::load_from_str(s).as_deref(),
        Ok([Yaml::String(back)]) if back == s
    )
}

fn literal_ok(s: &str) -> bool {
    if !s.contains('\n') || s.trim_matches('\n').is_empty() {
        return false;
    }
    if !s.chars().all(|c| c == '\n' || single_quotable(c)) {
        return false;
    }
    // A leading space on the first non-empty line would be read as
    // indentation.
    let first = s.split('\n').find(|l| !l.is_empty()).unwrap_or("");
    !first.starts_with(' ')
}
