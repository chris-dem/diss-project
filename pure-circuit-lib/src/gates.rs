use enum_derived::Rand;
use itertools::Itertools;
use macro_export::EnumCycle;
use misc_lib::EnumCycle;
use std::{
    fmt::Display,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Rand, PartialOrd, Ord, Default, EnumIter, Hash, EnumCycle,
)]
pub enum Value {
    #[default]
    Bot,
    Zero,
    One,
}

#[derive(Debug)]
pub enum ConversionError {
    OutOfRange,
    InvalidFormat,
    // Add other variants as needed
}


impl Value {
    fn inverse(&self) -> Self {
        match self {
            Self::One => Self::Zero,
            Self::Zero => Self::One,
            Self::Bot => Self::Bot,
        }
    }
}

pub trait GateCheck {
    fn check(&self, u: Value, v: Value, w: Option<Value>) -> Option<bool>;
}

pub trait RestrictedGateCheck {
    fn restricted_check(&self, u: Value, v: Value, w: Option<Value>) -> Option<bool>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, Hash, EnumCycle, Default, EnumIter)]
pub enum UnaryGate {
    #[default]
    Not,
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, Hash, EnumCycle, Default, EnumIter)]
pub enum BinaryGate {
    #[default]
    And,
    Or,
    Nor,
    Nand,
    Purify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, Hash)]
pub enum Gate {
    Unary(UnaryGate),
    Binary(BinaryGate),
}

impl IntoEnumIterator for Gate {
    type Iterator = std::vec::IntoIter<Self>;

    fn iter() -> Self::Iterator {
        UnaryGate::iter()
            .map(Self::Unary)
            .chain(BinaryGate::iter().map(Self::Binary))
            .collect_vec()
            .into_iter()
    }
}

impl Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unary(b) => write!(f, "{:?}", b),
            Self::Binary(b) => write!(f, "{:?}", b),
        }
    }
}

impl Default for Gate {
    fn default() -> Self {
        Self::Unary(Default::default())
    }
}

impl EnumCycle for Gate {
    fn toggle(&self) -> Self {
        match self {
            Self::Unary(UnaryGate::Not) => Self::Binary(BinaryGate::And),
            Self::Binary(BinaryGate::Purify) => Self::Unary(UnaryGate::Copy),
            Self::Binary(b) => Self::Binary(b.toggle()),
            Self::Unary(b) => Self::Unary(b.toggle()),
        }
    }
}

impl GateCheck for Gate {
    fn check(&self, u: Value, v: Value, w: Option<Value>) -> Option<bool> {
        match self {
            Gate::Unary(x) => x.check(u, v, w),
            Gate::Binary(x) => x.check(u, v, w),
        }
    }
}

impl GateCheck for BinaryGate {
    fn check(&self, u: Value, v: Value, w: Option<Value>) -> Option<bool> {
        let w = w?;
        Some(match self {
            BinaryGate::Or => match (u, v) {
                (Value::Zero, Value::Zero) => w == Value::Zero,
                (Value::One, _) | (_, Value::One) => w == Value::One,
                _ => true,
            },
            BinaryGate::And => match (u, v) {
                (Value::One, Value::One) => w == Value::One,
                (Value::Zero, _) | (_, Value::Zero) => w == Value::Zero,
                _ => true,
            },
            BinaryGate::Nor => match (u, v) {
                (Value::Zero, Value::Zero) => w == Value::One,
                (Value::One, _) | (_, Value::One) => w == Value::Zero,
                _ => true,
            },
            BinaryGate::Nand => match (u, v) {
                (Value::One, Value::One) => w == Value::Zero,
                (Value::Zero, _) | (_, Value::Zero) => w == Value::One,
                _ => true,
            },
            BinaryGate::Purify => match u {
                Value::Zero | Value::One => v == u && w == u,
                _ => v > Value::Bot || w > Value::Bot,
            },
        })
    }
}

impl GateCheck for UnaryGate {
    fn check(&self, u: Value, v: Value, w: Option<Value>) -> Option<bool> {
        if w.is_some() {
            return None;
        }
        Some(match self {
            UnaryGate::Not => match u {
                Value::One => v == Value::Zero,
                Value::Zero => v == Value::One,
                _ => true,
            },
            UnaryGate::Copy => match u {
                Value::One | Value::Zero => u == v,
                _ => true,
            },
        })
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NodeValue {
    GateNode(Gate),
    ValueNode(Value),
}

impl NodeValue {
    pub fn is_gate(&self) -> bool {
        if let Self::GateNode(_) = self {
            true
        } else {
            false
        }
    }
}

mod tests {
    use super::*;
    use itertools::{Itertools, assert_equal};

    #[test]
    fn test_gate_iter() {
        assert_equal(
            Gate::iter(),
            UnaryGate::iter()
                .map(Gate::Unary)
                .chain(BinaryGate::iter().map(Gate::Binary)),
        )
    }

    fn data_generator_uni() -> impl Iterator<Item = (Value, Value)> {
        Value::iter().cartesian_product(Value::iter())
    }

    fn data_generator_bin() -> impl Iterator<Item = (Value, Value, Value)> {
        data_generator_uni()
            .cartesian_product(Value::iter())
            .map(|((a, b), c)| (a, b, c))
    }

    mod unary_gates {
        use super::*;

        #[test]
        fn test_not() {
            for (u, v) in data_generator_uni() {
                let res = UnaryGate::Not.check(u, v, None);
                assert!(res.is_some());
                assert_eq!(
                    res,
                    BinaryGate::Nor.check(u, u, Some(v)),
                    "Value in {:?} Value out {:?}",
                    u,
                    v
                );
                assert_eq!(
                    res,
                    BinaryGate::Nand.check(u, u, Some(v)),
                    "Value in {:?} Value out {:?}",
                    u,
                    v
                );
            }
        }

        #[test]
        fn test_copy() {
            for (u, v) in data_generator_uni() {
                let res = UnaryGate::Copy.check(u, v, None);
                assert!(res.is_some());
                assert_eq!(res, BinaryGate::Or.check(u, u, Some(v)));
                assert_eq!(res, BinaryGate::And.check(u, u, Some(v)));
            }
        }
    }

    mod binary_gates {
        use super::*;

        #[test]
        fn test_and() {
            for (u, v, w) in data_generator_bin() {
                let res = BinaryGate::And.check(u, v, Some(w));
                assert!(res.is_some());
                assert_eq!(
                    res,
                    BinaryGate::Nor.check(u.inverse(), v.inverse(), Some(w))
                );
            }
        }

        #[test]
        fn test_nand() {
            for (u, v, w) in data_generator_bin() {
                let res = BinaryGate::Nand.check(u, v, Some(w));
                assert!(res.is_some());
                assert_eq!(res, BinaryGate::Or.check(u.inverse(), v.inverse(), Some(w)));
            }
        }

        #[test]
        fn test_or() {
            for (u, v, w) in data_generator_bin() {
                let res = BinaryGate::Or.check(u, v, Some(w));
                assert!(res.is_some());
                assert_eq!(
                    res,
                    BinaryGate::Nand.check(u.inverse(), v.inverse(), Some(w))
                );
            }
        }

        #[test]
        fn test_nor() {
            for (u, v, w) in data_generator_bin() {
                let res = BinaryGate::Nor.check(u, v, Some(w));
                assert!(res.is_some());
                assert_eq!(
                    res,
                    BinaryGate::And.check(u.inverse(), v.inverse(), Some(w))
                );
            }
        }

        #[test]
        fn test_purify() {
            for (u, v, w) in data_generator_bin() {
                let res = BinaryGate::Purify.check(u, v, Some(w));
                assert!(res.is_some());
                let cp1 = UnaryGate::Copy.check(u, v, None).unwrap();
                let cp2 = UnaryGate::Copy.check(u, w, None).unwrap();
                assert_eq!(
                    res,
                    Some(
                        (cp1 && cp2) && (u != Value::Bot || !(v == Value::Bot && w == Value::Bot))
                    ),
                    "Value in [{:?}] Value out [{:?}, {:?}]",
                    u,
                    v,
                    w
                );
            }
        }
    }
}
