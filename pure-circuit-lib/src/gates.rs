use enum_derived::Rand;
use macro_export::EnumCycle;
use misc_lib::EnumCycle;
use petgraph::Graph;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, Default, EnumIter, Hash, EnumCycle)]
pub enum Value {
    #[default]
    Bot,
    Zero,
    One,
}

impl Into<usize> for Value {
    fn into(self) -> usize {
        ((self as usize) + 1) % 3
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, Default, Hash)]
pub struct InformationOrdering(pub Value);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, Default, Hash)]
pub struct VoltageOrdering(pub Value);

impl PartialOrd for InformationOrdering {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.0, other.0) {
            (Value::Bot, Value::Zero) => Some(std::cmp::Ordering::Less),
            (Value::Zero, Value::Bot) => Some(std::cmp::Ordering::Greater),
            (Value::Bot, Value::One) => Some(std::cmp::Ordering::Less),
            (Value::One, Value::Bot) => Some(std::cmp::Ordering::Greater),
            _ => None,
        }
    }
}

impl PartialOrd for VoltageOrdering {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let fst = match self.0 {
            Value::Zero => 0,
            Value::Bot => 1,
            Value::One => 2,
        };

        let snd = match other.0 {
            Value::Zero => 0,
            Value::Bot => 1,
            Value::One => 2,
        };
        fst.partial_cmp(&snd)
    }
}

impl Ord for VoltageOrdering {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other)
            .expect("Should be `Some` for all instances")
    }
}

#[derive(Debug)]
pub enum ConversionError {
    OutOfRange,
    InvalidFormat,
    // Add other variants as needed
}

impl Value {
    fn inverse(self) -> Self {
        match self {
            Self::One => Self::Zero,
            Self::Zero => Self::One,
            Self::Bot => Self::Bot,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Rand, Hash, EnumCycle, Default, EnumIter)]
pub enum Gate {
    #[default]
    Copy,
    Not,
    And,
    Or,
    Nor,
    Nand,
    Purify,
}

impl Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateError {
    ArityError,
    NonDeterminsticGate,
}

impl Gate {
    pub fn arity(&self) -> (usize, usize) {
        match self {
            Gate::Copy => (1, 1),
            Gate::Not => (1, 1),
            Gate::Purify => (1, 2),
            _ => (2, 1),
        }
    }

    #[inline]
    fn check_arity(&self, in_vals: &[Value], out_vals: &[Value]) -> bool {
        let (ins, outs) = self.arity();
        in_vals.len() == ins && out_vals.len() == outs
    }

    fn apply(self, in_vals: &[Value]) -> Result<Value, GateError> {
        if in_vals.len() != self.arity().0 && self.arity().1 > 1 {
            return Err(GateError::ArityError);
        }
        match self {
            Self::Or => Ok(VoltageOrdering(in_vals[0])
                .max(VoltageOrdering(in_vals[1]))
                .0),
            Self::And => Ok(VoltageOrdering(in_vals[0])
                .min(VoltageOrdering(in_vals[1]))
                .0),
            Self::Copy => Ok(in_vals[0]),
            Self::Not => Ok(in_vals[0].inverse()),
            Self::Nor => Self::Or.apply(in_vals).map(Value::inverse),
            Self::Nand => Self::And.apply(in_vals).map(Value::inverse),
            Self::Purify => {
                log::warn!("Purify does not have a deterministic application");
                Err(GateError::NonDeterminsticGate)
            }
        }
    }

    pub fn check(&self, in_vals: &[Value], out_vals: &[Value]) -> Result<bool, GateError> {
        if !self.check_arity(in_vals, out_vals) {
            Err(GateError::ArityError)
        } else {
            match self {
                Self::Purify => Ok(match in_vals[0] {
                    Value::Bot => out_vals.contains(&Value::One) || out_vals.contains(&Value::Zero),
                    b => out_vals == [b, b],
                }),
                b if out_vals.len() == 1 => b.apply(in_vals).map(|val| val == out_vals[0]),
                b => unimplemented!("Gate {b:?} not implemented"),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GateStatus {
    Valid,
    InvalidArity,
    InvalidValues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Value,
    Gate,
}

pub trait NodeStateTrait: Debug + Clone + Copy + PartialEq + Eq + Hash {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NewNode;

impl NodeStateTrait for NewNode {}
impl NodeStateTrait for GateStatus {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeValue<I: NodeStateTrait> {
    ValueNode(Value),
    GateNode { gate: Gate, state_type: I },
}

impl<T: NodeStateTrait> NodeValue<T> {
    pub fn compare_types<I: NodeStateTrait>(&self, other: NodeValue<I>) -> bool {
        match (self, other) {
            (Self::GateNode { .. }, NodeValue::<I>::GateNode { .. }) => true,
            (Self::ValueNode(_), NodeValue::<I>::ValueNode(_)) => true,
            _ => false,
        }
    }
}

pub type NodeUnitialised = NodeValue<NewNode>;
pub type GraphNode = NodeValue<GateStatus>;

#[derive(Debug, Clone, Copy)]
pub struct GraphStruct<T> {
    pub node: GraphNode,
    pub additional_info: T,
}

impl<T> GraphStruct<T> {
    pub fn new(node: GraphNode, additional_info: T) -> Self {
        Self {
            node,
            additional_info,
        }
    }
    pub fn into_node(&self) -> GraphNode {
        self.node
    }
}

// impl Copy for GraphStruct<T> {

// }

impl NodeUnitialised {
    pub fn from_value(value: Value) -> Self {
        Self::ValueNode(value)
    }

    pub fn from_gate(gate: Gate) -> Self {
        Self::GateNode {
            gate: gate,
            state_type: NewNode,
        }
    }
}

impl<T: NodeStateTrait> EnumCycle for NodeValue<T> {
    fn toggle(&self) -> Self {
        match *self {
            Self::GateNode { gate, state_type } => Self::GateNode {
                gate: gate.toggle(),
                state_type,
            },
            Self::ValueNode(b) => Self::ValueNode(b.toggle()),
        }
    }
}

impl<T: NodeStateTrait> NodeValue<T> {
    pub fn is_gate(&self) -> bool {
        matches!(
            self,
            Self::GateNode {
                gate: _,
                state_type: _
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::enum_strategy;
    use itertools::Itertools;
    use proptest::prelude::*;
    use strum::IntoEnumIterator;

    #[allow(dead_code)]
    fn data_generator_uni() -> impl Iterator<Item = (Value, Value)> {
        Value::iter().cartesian_product(Value::iter())
    }

    #[allow(dead_code)]
    fn data_generator_bin() -> impl Iterator<Item = (Value, Value, Value)> {
        data_generator_uni()
            .cartesian_product(Value::iter())
            .map(|((a, b), c)| (a, b, c))
    }

    mod errors {
        use super::*;

        #[test]
        fn test_nondeterministic() {
            for gate in Gate::iter() {
                let inps = [Value::Bot, Value::Bot, Value::Bot];
                let (s1, s2) = gate.arity();
                let (inps, _) = (&inps[0..s1], &inps[s1..s1 + s2]);
                let res = gate.apply(inps);
                if gate != Gate::Purify {
                    assert!(res.is_ok());
                } else {
                    assert_eq!(res, Err(GateError::NonDeterminsticGate));
                }
            }
        }

        proptest! {
            #[test]
            fn test_arities(
                gate in enum_strategy::<Gate>(),
                in_values in proptest::collection::vec(
                    enum_strategy::<Value>(),
                    0..=15
                ),
                out_values in proptest::collection::vec(
                    enum_strategy::<Value>(),
                    0..=15
                )
            ) {
                if gate.check_arity(&in_values, &out_values) {
                    prop_assert_ne!(gate.check(&in_values, &out_values), Err(GateError::ArityError));
                } else {
                    prop_assert_eq!(gate.check(&in_values, &out_values), Err(GateError::ArityError));
                }
            }
        }
    }

    mod unary_gates {
        #[allow(unused_imports)]
        use super::*;

        #[test]
        fn test_not() {
            for (u, v) in data_generator_uni() {
                let res = Gate::Not.check(&[u], &[v]);
                assert!(res.is_ok());
                assert_eq!(
                    res,
                    Gate::Nor.check(&[u, u], &[v]),
                    "Value in {:?} Value out {:?}",
                    u,
                    v
                );
                assert_eq!(
                    res,
                    Gate::Nand.check(&[u, u], &[v]),
                    "Value in {:?} Value out {:?}",
                    u,
                    v
                );
            }
        }

        #[test]
        fn test_copy() {
            for (u, v) in data_generator_uni() {
                let res = Gate::Copy.check(&[u], &[v]);
                assert!(res.is_ok());
                assert_eq!(res, Gate::Or.check(&[u, u], &[v]));
                assert_eq!(res, Gate::And.check(&[u, u], &[v]));
            }
        }
    }

    mod binary_gates {
        #[allow(unused_imports)]
        use super::*;

        #[test]
        fn test_and() {
            for (u, v, w) in data_generator_bin() {
                let res = Gate::And.check(&[u, v], &[w]);
                assert!(res.is_ok());
                assert_eq!(res, Gate::Nor.check(&[u.inverse(), v.inverse()], &[w]));
            }
        }

        #[test]
        fn test_nand() {
            for (u, v, w) in data_generator_bin() {
                let res = Gate::Nand.check(&[u, v], &[w]);
                assert!(res.is_ok());
                assert_eq!(res, Gate::Or.check(&[u.inverse(), v.inverse()], &[w]));
            }
        }

        #[test]
        fn test_or() {
            for (u, v, w) in data_generator_bin() {
                let res = Gate::Or.check(&[u, v], &[w]);
                assert!(res.is_ok());
                assert_eq!(res, Gate::Nand.check(&[u.inverse(), v.inverse()], &[w]));
            }
        }

        #[test]
        fn test_nor() {
            for (u, v, w) in data_generator_bin() {
                let res = Gate::Nor.check(&[u, v], &[w]);
                assert!(res.is_ok(), "{:?}", res);
                assert_eq!(res, Gate::And.check(&[u.inverse(), v.inverse()], &[w]));
            }
        }

        #[test]
        fn test_purify() {
            for (u, v, w) in data_generator_bin() {
                let res = Gate::Purify.check(&[u], &[v, w]);
                assert!(res.is_ok());
                let cp1 = Gate::Copy.check(&[u], &[v]).unwrap();
                let cp2 = Gate::Copy.check(&[u], &[w]).unwrap();
                assert_eq!(
                    res,
                    Ok((u != Value::Bot && cp1 && cp2)
                        || (u == Value::Bot && ((v != Value::Bot) || (w != Value::Bot)))),
                    "Value in [{:?}] Value out [{:?}, {:?}]",
                    u,
                    v,
                    w
                );
            }
        }
    }
}
