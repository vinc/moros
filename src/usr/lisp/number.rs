use super::Err;
use crate::could_not;

use alloc::format;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt;
use core::num::ParseIntError;
use core::ops::{Add, Div, Mul, Neg, Rem, Shl, Shr, Sub};
use core::str::FromStr;
use num_bigint::BigInt;
use num_bigint::ParseBigIntError;
use num_traits::cast::ToPrimitive;
use num_traits::Num;
use num_traits::Zero;

#[derive(Clone, PartialEq)]
pub enum Number {
    BigInt(BigInt),
    Float(f64),
    Int(i64),
}

macro_rules! trigonometric_method {
    ($op:ident) => {
        pub fn $op(&self) -> Number {
            Number::Float(libm::$op(self.into()))
        }
    };
}

macro_rules! arithmetic_method {
    ($op:ident, $checked_op:ident) => {
        pub fn $op(self, other: Number) -> Number {
            match (self, other) {
                (Number::BigInt(a), Number::BigInt(b)) => {
                    Number::BigInt(a.$op(b))
                }
                (Number::BigInt(a), Number::Int(b)) => {
                    Number::BigInt(a.$op(b))
                }
                (Number::Int(a), Number::BigInt(b)) => {
                    Number::BigInt(a.$op(b))
                }
                (Number::Int(a), Number::Int(b)) => {
                    if let Some(r) = a.$checked_op(b) {
                        Number::Int(r)
                    } else {
                        Number::BigInt(BigInt::from(a).$op(BigInt::from(b)))
                    }
                }
                (Number::Int(a), Number::Float(b)) => {
                    Number::Float((a as f64).$op(b))
                }
                (Number::Float(a), Number::Int(b)) => {
                    Number::Float(a.$op(b as f64))
                }
                (Number::Float(a), Number::Float(b)) => {
                    Number::Float(a.$op(b))
                }
                _ => {
                    Number::Float(f64::NAN) // TODO
                }
            }
        }
    };
}

impl Number {
    trigonometric_method!(cos);
    trigonometric_method!(sin);
    trigonometric_method!(tan);
    trigonometric_method!(acos);
    trigonometric_method!(asin);
    trigonometric_method!(atan);

    arithmetic_method!(add, checked_add);
    arithmetic_method!(sub, checked_sub);
    arithmetic_method!(mul, checked_mul);
    arithmetic_method!(div, checked_div);

    // NOTE: Rem use `libm::fmod` for `f64` instead of `rem`
    pub fn rem(self, other: Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::BigInt(b)) => {
                Number::BigInt(a.rem(b))
            }
            (Number::BigInt(a), Number::Int(b)) => {
                Number::BigInt(a.rem(b))
            }
            (Number::Int(a), Number::BigInt(b)) => {
                Number::BigInt(a.rem(b))
            }
            (Number::Int(a), Number::Int(b)) => {
                if let Some(r) = a.checked_rem(b) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(a).rem(BigInt::from(b)))
                }
            }
            (Number::Int(a), Number::Float(b)) => {
                Number::Float(libm::fmod(a as f64, b))
            }
            (Number::Float(a), Number::Int(b)) => {
                Number::Float(libm::fmod(a, b as f64))
            }
            (Number::Float(a), Number::Float(b)) => {
                Number::Float(libm::fmod(a, b))
            }
            _ => {
                Number::Float(f64::NAN) // TODO
            }
        }
    }

    pub fn pow(&self, other: &Number) -> Number {
        let bmax = BigInt::from(u32::MAX);
        let imax = u32::MAX as i64;
        match (self, other) {
            (_, Number::BigInt(b)) if *b > bmax => {
                Number::Float(f64::INFINITY)
            }
            (_, Number::Int(b)) if *b > imax => {
                Number::Float(f64::INFINITY)
            }
            (Number::BigInt(a), Number::Int(b)) => {
                Number::BigInt(a.pow(*b as u32))
            }
            (Number::Int(a), Number::Int(b)) => {
                if let Some(r) = a.checked_pow(*b as u32) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(*a)).pow(other)
                }
            }
            (Number::Int(a), Number::Float(b)) => {
                Number::Float(libm::pow(*a as f64, *b))
            }
            (Number::Float(a), Number::Int(b)) => {
                Number::Float(libm::pow(*a, *b as f64))
            }
            (Number::Float(a), Number::Float(b)) => {
                Number::Float(libm::pow(*a, *b))
            }
            _ => {
                Number::Float(f64::NAN) // TODO
            }
        }
    }

    pub fn neg(self) -> Number {
        match self {
            Number::BigInt(a) => {
                Number::BigInt(-a)
            }
            Number::Int(a) => {
                if let Some(r) = a.checked_neg() {
                    Number::Int(r)
                } else {
                    Number::BigInt(-BigInt::from(a))
                }
            }
            Number::Float(a) => {
                Number::Float(-a)
            }
        }
    }

    pub fn trunc(self) -> Number {
        if let Number::Float(a) = self {
            Number::Int(libm::trunc(a) as i64)
        } else {
            self
        }
    }

    pub fn shl(self, other: Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::Int(b)) => {
                Number::BigInt(a.shl(b))
            }
            (Number::Int(a), Number::Int(b)) => {
                if let Some(r) = a.checked_shl(b as u32) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(a).shl(b))
                }
            }
            _ => {
                Number::Float(f64::NAN) // TODO
            }
        }
    }

    pub fn shr(self, other: Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::Int(b)) => {
                Number::BigInt(a.shr(b))
            }
            (Number::Int(a), Number::Int(b)) => {
                if let Some(r) = a.checked_shr(b as u32) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(a).shr(b))
                }
            }
            _ => {
                Number::Float(f64::NAN) // TODO
            }
        }
    }

    pub fn to_be_bytes(&self) -> Vec<u8> {
        match self {
            Number::Int(n) => n.to_be_bytes().to_vec(),
            Number::Float(n) => n.to_be_bytes().to_vec(),
            Number::BigInt(n) => n.to_bytes_be().1, // TODO
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Number::Int(n) => *n == 0,
            Number::Float(n) => *n == 0.0,
            Number::BigInt(n) => n.is_zero(),
        }
    }
}

impl Neg for Number {
    type Output = Number;
    fn neg(self) -> Number {
        self.neg()
    }
}

macro_rules! operator {
    ($t:ty, $op:ident) => {
        impl $t for Number {
            type Output = Number;
            fn $op(self, other: Number) -> Number {
                self.$op(other)
            }
        }
    };
}

operator!(Add, add);
operator!(Sub, sub);
operator!(Mul, mul);
operator!(Div, div);
operator!(Rem, rem);
operator!(Shl, shl);
operator!(Shr, shr);

use core::cmp::Ordering;

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => {
                a.partial_cmp(b)
            }
            (Number::Float(a), Number::Float(b)) => {
                a.partial_cmp(b)
            }
            (Number::BigInt(a), Number::BigInt(b)) => {
                a.partial_cmp(b)
            }
            (Number::Int(a), Number::Float(b)) => {
                (*a as f64).partial_cmp(b)
            }
            (Number::Int(a), Number::BigInt(b)) => {
                (*a as f64).partial_cmp(&b.to_f64().unwrap())
            }
            (Number::Float(a), Number::Int(b)) => {
                a.partial_cmp(&(*b as f64))
            }
            (Number::Float(a), Number::BigInt(b)) => {
                a.partial_cmp(&b.to_f64().unwrap())
            }
            (Number::BigInt(a), Number::Float(b))  => {
                a.to_f64().unwrap().partial_cmp(b)
            }
            (Number::BigInt(a), Number::Int(b)) => {
                a.to_f64().unwrap().partial_cmp(&(*b as f64))
            }
        }
    }
}

fn parse_int(s: &str) -> Result<i64, ParseIntError> {
    if s.starts_with("0x") {
        i64::from_str_radix(&s[2..], 16)
    } else if s.starts_with("-0x") {
        i64::from_str_radix(&s[3..], 16).map(|n| -n)
    } else if s.starts_with("0b") {
        i64::from_str_radix(&s[2..], 2)
    } else if s.starts_with("-0b") {
        i64::from_str_radix(&s[3..], 2).map(|n| -n)
    } else {
        i64::from_str_radix(s, 10)
    }
}

fn parse_bigint(s: &str) -> Result<BigInt, ParseBigIntError> {
    if s.starts_with("0x") {
        BigInt::from_str_radix(&s[2..], 16)
    } else if s.starts_with("-0x") {
        BigInt::from_str_radix(&s[3..], 16).map(|n| -n)
    } else if s.starts_with("0b") {
        BigInt::from_str_radix(&s[2..], 2)
    } else if s.starts_with("-0b") {
        BigInt::from_str_radix(&s[3..], 2).map(|n| -n)
    } else {
        BigInt::from_str_radix(s, 10)
    }
}

impl FromStr for Number {
    type Err = Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = could_not!("parse number");
        if s.is_empty() {
            Ok(Number::Int(0))
        } else if s.contains('.') {
            if let Ok(n) = s.parse() {
                Ok(Number::Float(n))
            } else {
                err
            }
        } else if let Ok(n) = parse_int(s) {
            Ok(Number::Int(n))
        } else if let Ok(n) = parse_bigint(s) {
            Ok(Number::BigInt(n))
        } else {
            err
        }
    }
}

impl From<&str> for Number {
    fn from(s: &str) -> Self {
        if let Ok(num) = s.parse() {
            num
        } else {
            Number::Float(f64::NAN)
        }
    }
}

impl From<f64> for Number {
    fn from(num: f64) -> Self {
        Number::Float(num)
    }
}

impl From<u8> for Number {
    fn from(num: u8) -> Self {
        Number::Int(num as i64)
    }
}

impl From<usize> for Number {
    fn from(num: usize) -> Self {
        if num > i64::MAX as usize {
            Number::BigInt(BigInt::from(num))
        } else {
            Number::Int(num as i64)
        }
    }
}

impl From<&Number> for f64 {
    fn from(num: &Number) -> f64 {
        match num {
            Number::Float(n) => *n,
            Number::Int(n) => *n as f64,
            Number::BigInt(n) => n.to_f64().unwrap_or(f64::NAN),
        }
    }
}

macro_rules! try_from_number {
    ($int:ident, $to_int:ident) => {
        impl TryFrom<Number> for $int {
            type Error = Err;

            fn try_from(num: Number) -> Result<Self, Self::Error> {
                let err = Err::Reason(
                    format!("Expected an integer between 0 and {}", $int::MAX)
                );
                match num {
                    Number::Float(n) => $int::try_from(n as i64).or(Err(err)),
                    Number::Int(n) => $int::try_from(n).or(Err(err)),
                    Number::BigInt(n) => n.$to_int().ok_or(err),
                }
            }
        }
    };
}

try_from_number!(usize, to_usize);
try_from_number!(u32, to_u32);
try_from_number!(u8, to_u8);

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME: alloc error
        // write!(f, "{}", self)
        match self {
            Number::Int(n) => {
                write!(f, "{}", n)
            }
            Number::BigInt(n) => {
                // FIXME: rust-lld: error: undefined symbol: fmod
                // write!(f, "{}", n)
                let mut v = Vec::new();
                let mut n = n.clone();
                if n < BigInt::from(0) {
                    write!(f, "-").ok();
                    n = -n;
                }
                loop {
                    v.push((n.clone() % BigInt::from(10)).to_u64().unwrap());
                    n /= BigInt::from(10);
                    if n == BigInt::from(0) {
                        break;
                    }
                }
                for d in v.iter().rev() {
                    write!(f, "{}", d).ok();
                }
                Ok(())
            }
            Number::Float(n) => {
                if n - libm::trunc(*n) == 0.0 {
                    write!(f, "{}.0", n)
                } else {
                    write!(f, "{}", n)
                }
            }
        }
    }
}
