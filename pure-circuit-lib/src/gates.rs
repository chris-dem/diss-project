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

trait GateCheck {
    fn check(&self, u: Value, v: Value, w: Option<Value>) -> bool;
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
    fn check(&self, _u: Value, _v: Value, _w: Option<Value>) -> bool {
        todo!("Not implemented");
    }
}

impl GateCheck for BinaryGate {
    fn check(&self, _u: Value, _v: Value, _w: Option<Value>) -> bool {
        match self {
            BinaryGate::Or => todo!(),
            BinaryGate::And => todo!(),
            BinaryGate::Nor => todo!(),
            BinaryGate::Nand => todo!(),
            BinaryGate::Purify => todo!(),
        }
    }
}

impl GateCheck for UnaryGate {
    fn check(&self, _u: Value, _v: Value, _w: Option<Value>) -> bool {
        todo!("Not implemented");
    }
}

mod tests {
    use super::*;
    use itertools::Itertools;
    mod binary_gates {
        use super::*;
        fn data_generator_uni() -> impl Iterator<Item = (Value, Value)> {
            Value::iter().cartesian_product(Value::iter())
        }

        fn data_generator_bin() -> impl Iterator<Item = (Value, Value, Value)> {
            data_generator_uni()
                .cartesian_product(Value::iter())
                .map(|((a, b), c)| (a, b, c))
        }

        #[test]
        fn test_and() {
            for (u, v, w) in data_generator_bin() {
                let expected = match (u, v) {
                    (Value::One, Value::One) => w == Value::One,
                    (Value::Zero, _) | (_, Value::Zero) => w == Value::Zero,
                    _ => true,
                };
                assert_eq!(BinaryGate::And.check(u, v, Some(w)), expected)
            }
        }

        #[test]
        fn test_nand() {
            for (u, v, w) in data_generator_bin() {
                let expected = match (u, v) {
                    (Value::One, Value::One) => w == Value::Zero,
                    (Value::Zero, _) | (_, Value::Zero) => w == Value::One,
                    _ => true,
                };
                assert_eq!(BinaryGate::Nand.check(u, v, Some(w)), expected)
            }
        }

        #[test]
        fn test_or() {
            for (u, v, w) in data_generator_bin() {
                let expected = match (u, v) {
                    (Value::Zero, Value::Zero) => w == Value::Zero,
                    (Value::One, _) | (_, Value::One) => w == Value::One,
                    _ => true,
                };
                assert_eq!(BinaryGate::Or.check(u, v, Some(w)), expected)
            }
        }

        #[test]
        fn test_nor() {
            for (u, v, w) in data_generator_bin() {
                let expected = match (u, v) {
                    (Value::Zero, Value::Zero) => w == Value::One,
                    (Value::One, _) | (_, Value::One) => w == Value::Zero,
                    _ => true,
                };
                assert_eq!(BinaryGate::Nor.check(u, v, Some(w)), expected)
            }
        }

        #[test]
        fn test_purify() {
            for (u, v, w) in data_generator_bin() {
                let expected = match u {
                    Value::Zero | Value::One => v == u && w == u,
                    _ => v > Value::Bot || w > Value::Bot,
                };
                assert_eq!(BinaryGate::Purify.check(u, v, Some(w)), expected)
            }
        }
    }
}
