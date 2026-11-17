use serde::de::{Deserialize, Deserializer, Visitor};
use serde::{Serialize, Serializer};
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};

/// A YAML number: an unsigned integer, a negative integer, or a float.
///
/// # Examples
///
/// ```rust
/// use serde_yaml_rust2::Number;
///
/// let n = Number::from(42u64);
/// assert!(n.is_u64() && n.is_i64() && !n.is_f64());
/// assert_eq!(n.as_i64(), Some(42));
/// ```
#[derive(Clone, Copy)]
pub struct Number {
    n: N,
}

#[derive(Clone, Copy, Debug)]
enum N {
    PosInt(u64),
    NegInt(i64),
    Float(f64),
}

impl Number {
    /// Returns `true` if the number is an integer that fits in an `i64`.
    pub fn is_i64(&self) -> bool {
        match self.n {
            N::PosInt(v) => i64::try_from(v).is_ok(),
            N::NegInt(_) => true,
            N::Float(_) => false,
        }
    }

    /// Returns `true` if the number is a non-negative integer (fits in a `u64`).
    pub fn is_u64(&self) -> bool {
        matches!(self.n, N::PosInt(_))
    }

    /// Returns `true` if the number is stored as a floating point value.
    pub fn is_f64(&self) -> bool {
        matches!(self.n, N::Float(_))
    }

    /// Returns the number as an `i64` if it is an integer in range, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_yaml_rust2::Number;
    ///
    /// assert_eq!(Number::from(-5i64).as_i64(), Some(-5));
    /// assert_eq!(Number::from(1.5f64).as_i64(), None);
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match self.n {
            N::PosInt(v) => i64::try_from(v).ok(),
            N::NegInt(v) => Some(v),
            N::Float(_) => None,
        }
    }

    /// Returns the number as a `u64` if it is a non-negative integer, otherwise `None`.
    pub fn as_u64(&self) -> Option<u64> {
        match self.n {
            N::PosInt(v) => Some(v),
            N::NegInt(_) | N::Float(_) => None,
        }
    }

    /// Returns the number as an `f64`. Integers are converted, possibly losing precision.
    pub fn as_f64(&self) -> Option<f64> {
        match self.n {
            N::PosInt(v) => Some(v as f64),
            N::NegInt(v) => Some(v as f64),
            N::Float(v) => Some(v),
        }
    }

    /// Returns `true` if the number is a floating point NaN (`.nan` in YAML).
    pub fn is_nan(&self) -> bool {
        matches!(self.n, N::Float(f) if f.is_nan())
    }

    /// Returns `true` if the number is positive or negative infinity (`.inf` / `-.inf`).
    pub fn is_infinite(&self) -> bool {
        matches!(self.n, N::Float(f) if f.is_infinite())
    }

    /// Returns `true` unless the number is a NaN or infinite float. Integers are always finite.
    pub fn is_finite(&self) -> bool {
        match self.n {
            N::Float(f) => f.is_finite(),
            _ => true,
        }
    }

    pub(crate) fn from_f64_str(s: &str) -> Number {
        let t = s.trim();
        let f = match t {
            ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => f64::INFINITY,
            "-.inf" | "-.Inf" | "-.INF" => f64::NEG_INFINITY,
            ".nan" | ".NaN" | ".NAN" => f64::NAN,
            _ => t.parse::<f64>().unwrap_or(f64::NAN),
        };
        Number::from(f)
    }

    pub(crate) fn deserialize_any<'de, V, E>(&self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
        E: serde::de::Error,
    {
        match self.n {
            N::PosInt(v) => visitor.visit_u64(v),
            N::NegInt(v) => visitor.visit_i64(v),
            N::Float(v) => visitor.visit_f64(v),
        }
    }
}

/// Creates a `Number` from an `i64`.
impl From<i64> for Number {
    fn from(v: i64) -> Self {
        let n = if v < 0 {
            N::NegInt(v)
        } else {
            N::PosInt(v as u64)
        };
        Number { n }
    }
}

/// Creates a `Number` from a `u64`.
impl From<u64> for Number {
    fn from(v: u64) -> Self {
        Number { n: N::PosInt(v) }
    }
}

/// Creates a `Number` from an `f64`.
impl From<f64> for Number {
    fn from(v: f64) -> Self {
        Number { n: N::Float(v) }
    }
}

macro_rules! from_signed {
    ($($t:ty),*) => {$(
        /// Creates a `Number` from a signed integer type.
        impl From<$t> for Number {
            fn from(v: $t) -> Self {
                Number::from(v as i64)
            }
        }
    )*};
}

macro_rules! from_unsigned {
    ($($t:ty),*) => {$(
        /// Creates a `Number` from an unsigned integer type.
        impl From<$t> for Number {
            fn from(v: $t) -> Self {
                Number::from(v as u64)
            }
        }
    )*};
}

from_signed!(i8, i16, i32, isize);
from_unsigned!(u8, u16, u32, usize);

/// Creates a `Number` from an `f32`, using its shortest decimal representation so `0.1f32` becomes `0.1`.
impl From<f32> for Number {
    fn from(v: f32) -> Self {
        let wide = if v.is_finite() {
            ryu::Buffer::new()
                .format_finite(v)
                .parse::<f64>()
                .unwrap_or(f64::from(v))
        } else {
            f64::from(v)
        };
        Number::from(wide)
    }
}

/// Formats the number as YAML: `.nan`, `.inf` and `-.inf` for special floats.
impl Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.n {
            N::PosInt(v) => f.write_str(itoa::Buffer::new().format(v)),
            N::NegInt(v) => f.write_str(itoa::Buffer::new().format(v)),
            N::Float(v) => {
                if v.is_nan() {
                    f.write_str(".nan")
                } else if v.is_infinite() {
                    f.write_str(if v.is_sign_negative() {
                        "-.inf"
                    } else {
                        ".inf"
                    })
                } else {
                    f.write_str(ryu::Buffer::new().format_finite(v))
                }
            }
        }
    }
}

/// Formats as `Number(<value>)`.
impl Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Number({})", self)
    }
}

/// Numbers are equal only if they have the same kind and value; floats compare by bit pattern, so NaN equals NaN.
impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self.n, other.n) {
            (N::PosInt(a), N::PosInt(b)) => a == b,
            (N::NegInt(a), N::NegInt(b)) => a == b,
            (N::Float(a), N::Float(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}

/// Total equality, since floats compare by bits.
impl Eq for Number {}

/// Hashes consistently with equality.
impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.n {
            N::PosInt(v) => {
                state.write_u8(0);
                v.hash(state);
            }
            N::NegInt(v) => {
                state.write_u8(1);
                v.hash(state);
            }
            N::Float(v) => {
                state.write_u8(2);
                v.to_bits().hash(state);
            }
        }
    }
}

/// Serializes as a `u64`, `i64` or `f64`.
impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.n {
            N::PosInt(v) => serializer.serialize_u64(v),
            N::NegInt(v) => serializer.serialize_i64(v),
            N::Float(v) => serializer.serialize_f64(v),
        }
    }
}

struct NumberVisitor;

impl<'de> Visitor<'de> for NumberVisitor {
    type Value = Number;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a number")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Number, E> {
        Ok(Number::from(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Number, E> {
        Ok(Number::from(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Number, E> {
        Ok(Number::from(v))
    }
}

/// Deserializes any numeric input.
impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NumberVisitor)
    }
}
