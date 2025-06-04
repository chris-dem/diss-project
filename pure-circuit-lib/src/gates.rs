use enum_derived::Rand;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, PartialOrd, Ord, Default, EnumIter)]
pub(crate) enum Value {
    #[default]
    Bot,
    Zero,
    One,
}

impl Value {
    fn inverse(&self) -> Value {
        match self {
            Value::One => Value::Zero,
            Value::Zero => Value::One,
            Value::Bot => Value::Bot,
        }
    }
}

pub(crate) trait GateCheck {
    fn check(&self, u: Value, v: Value, w: Option<Value>) -> Option<bool>;
}

pub(crate) trait RestrictedGateCheck {
    fn restricted_check(&self, u: Value, v: Value, w: Option<Value>) -> Option<bool>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand)]
pub(crate) enum UnaryGate {
    Not,
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand)]
pub(crate) enum BinaryGate {
    And,
    Or,
    Nor,
    Nand,
    Purify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand)]
pub(crate) enum Gate {
    Unary(UnaryGate),
    Binary(BinaryGate),
}

impl GateCheck for Gate {
    fn check(&self, u: Value, v: Value, w: Option<Value>) -> Option<bool> {
        match self {
            Gate::Unary(x) =>  x.check(u, v, w),
            Gate::Binary(x) =>  x.check(u, v, w),
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

mod tests {
    use super::*;
    use itertools::Itertools;

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
