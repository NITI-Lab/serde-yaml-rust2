use serde::de::{Deserialize, Deserializer, Visitor};
use serde::{Serialize, Serializer};
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};

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
    pub fn is_i64(&self) -> bool {
        match self.n {
            N::PosInt(v) => i64::try_from(v).is_ok(),
            N::NegInt(_) => true,
            N::Float(_) => false,
        }
    }

    pub fn is_u64(&self) -> bool {
        matches!(self.n, N::PosInt(_))
    }

    pub fn is_f64(&self) -> bool {
        matches!(self.n, N::Float(_))
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self.n {
            N::PosInt(v) => i64::try_from(v).ok(),
            N::NegInt(v) => Some(v),
            N::Float(_) => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self.n {
            N::PosInt(v) => Some(v),
            N::NegInt(_) | N::Float(_) => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.n {
            N::PosInt(v) => Some(v as f64),
            N::NegInt(v) => Some(v as f64),
            N::Float(v) => Some(v),
        }
    }

    pub fn is_nan(&self) -> bool {
        matches!(self.n, N::Float(f) if f.is_nan())
    }

    pub fn is_infinite(&self) -> bool {
        matches!(self.n, N::Float(f) if f.is_infinite())
    }

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

impl From<u64> for Number {
    fn from(v: u64) -> Self {
        Number { n: N::PosInt(v) }
    }
}

impl From<f64> for Number {
    fn from(v: f64) -> Self {
        Number { n: N::Float(v) }
    }
}

macro_rules! from_signed {
    ($($t:ty),*) => {$(
        impl From<$t> for Number {
            fn from(v: $t) -> Self {
                Number::from(v as i64)
            }
        }
    )*};
}

macro_rules! from_unsigned {
    ($($t:ty),*) => {$(
        impl From<$t> for Number {
            fn from(v: $t) -> Self {
                Number::from(v as u64)
            }
        }
    )*};
}

from_signed!(i8, i16, i32, isize);
from_unsigned!(u8, u16, u32, usize);

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

impl Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.n {
            N::PosInt(v) => f.write_str(itoa::Buffer::new().format(v)),
            N::NegInt(v) => f.write_str(itoa::Buffer::new().format(v)),
            N::Float(v) => {
                if v.is_nan() {
                    f.write_str(".nan")
                } else if v.is_infinite() {
                    f.write_str(if v.is_sign_negative() { "-.inf" } else { ".inf" })
                } else {
                    f.write_str(ryu::Buffer::new().format_finite(v))
                }
            }
        }
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Number({})", self)
    }
}

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

impl Eq for Number {}

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

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NumberVisitor)
    }
}
