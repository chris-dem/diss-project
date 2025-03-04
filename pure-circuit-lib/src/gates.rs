use enum_derived::Rand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, PartialOrd, Ord, Default)]
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
    Copy,
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

mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_and() {
        }

    }
}
