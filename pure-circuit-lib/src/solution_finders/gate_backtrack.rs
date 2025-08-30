use crate::{
    gates::{Gate, Value, VoltageOrdering},
    solution_finders::backtracking::BitString,
};
use anyhow::{Result as ARes, anyhow};

use crate::solution_finders::base_finder::MAX_DEGREE;

type BitStringIOState = (
    [Option<BitString>; MAX_DEGREE],
    [Option<BitString>; MAX_DEGREE],
);

impl Gate {
    pub(crate) fn set_value(
        &self,
        assingment_input: [Option<Value>; MAX_DEGREE],
        assignment_output: [Option<Value>; MAX_DEGREE],
    ) -> ARes<BitStringIOState> {
        #[allow(unreachable_code)]
        match self {
            Self::Not => match (assingment_input, assignment_output) {
                ([Some(v), _], [None, _]) => Ok((
                    [Some(BitString::from(v)), None],
                    [Some(BitString::from(v.inverse())), None],
                )),
                ([None, _], [Some(v), _]) => Ok((
                    [Some(BitString::from(v.inverse())), None],
                    [Some(BitString::from(v)), None],
                )),
                ([Some(a), _], [Some(b), _]) if a == b.inverse() => {
                    Ok(([Some(a.into()), None], [Some(b.into()), None]))
                }
                _ => ARes::Err(anyhow!("Incorrect assingment for {self}")),
            },
            Self::Copy => match (assingment_input, assignment_output) {
                ([Some(v), _], [None, _]) | ([None, _], [Some(v), _]) => Ok((
                    [Some(BitString::from(v)), None],
                    [Some(BitString::from(v)), None],
                )),
                ([Some(a), _], [Some(b), _]) if a == b => {
                    Ok(([Some(a.into()), None], [Some(b.into()), None]))
                }
                _ => ARes::Err(anyhow!("Incorrect assingment for {self}")),
            },
            Self::And => match (assingment_input, assignment_output) {
                ([None, None], [Some(v), _]) => Ok((
                    [
                        Some(BitString::greater_than(v)),
                        Some(BitString::greater_than(v)),
                    ],
                    [Some(v.into()), None],
                )),
                ([None, Some(v)], [None, _]) => Ok((
                    [Some(BitString::all()), Some(v.into())],
                    [Some(BitString::smaller_than(v)), None],
                )),
                ([Some(v), None], [None, _]) => Ok((
                    [Some(v.into()), Some(BitString::all())],
                    [Some(BitString::smaller_than(v)), None],
                )),
                ([None, Some(a)], [Some(v), _]) if VoltageOrdering(v) <= VoltageOrdering(a) => {
                    Ok((
                        [
                            Some(if a == v {
                                BitString::greater_than(a)
                            } else {
                                v.into()
                            }),
                            Some(a.into()),
                        ],
                        [Some(v.into()), None],
                    ))
                }
                ([Some(a), None], [Some(v), _]) if VoltageOrdering(v) <= VoltageOrdering(a) => {
                    Ok((
                        [
                            Some(a.into()),
                            Some(if a == v {
                                BitString::greater_than(a)
                            } else {
                                v.into()
                            }),
                        ],
                        [Some(v.into()), None],
                    ))
                }
                ([Some(a), Some(b)], [None, _]) => Ok((
                    [Some(a.into()), Some(b.into())],
                    [Some(Gate::And.apply(&[a, b]).unwrap().into()), None],
                )),
                ([Some(a), Some(b)], [Some(v), _])
                    if Gate::And.check(&[a, b], &[v]).is_ok_and(|x| x) =>
                {
                    Ok(([Some(a.into()), Some(b.into())], [Some(v.into()), None]))
                }
                _ => ARes::Err(anyhow!("Incorrect assingment for {self}")),
            },
            Self::Or => match (assingment_input, assignment_output) {
                ([None, None], [Some(v), _]) => Ok((
                    [
                        Some(BitString::smaller_than(v)),
                        Some(BitString::smaller_than(v)),
                    ],
                    [Some(v.into()), None],
                )),
                ([None, Some(v)], [None, _]) => Ok((
                    [Some(BitString::all()), Some(v.into())],
                    [Some(BitString::greater_than(v)), None],
                )),
                ([Some(v), None], [None, _]) => Ok((
                    [Some(v.into()), Some(BitString::all())],
                    [Some(BitString::greater_than(v)), None],
                )),
                ([None, Some(a)], [Some(v), _]) if VoltageOrdering(a) <= VoltageOrdering(v) => {
                    Ok((
                        [
                            Some(if a == v {
                                BitString::smaller_than(a)
                            } else {
                                v.into()
                            }),
                            Some(a.into()),
                        ],
                        [Some(v.into()), None],
                    ))
                }
                ([Some(a), None], [Some(v), _]) if VoltageOrdering(a) <= VoltageOrdering(v) => {
                    Ok((
                        [
                            Some(a.into()),
                            Some(if a == v {
                                BitString::smaller_than(a)
                            } else {
                                v.into()
                            }),
                        ],
                        [Some(v.into()), None],
                    ))
                }
                ([Some(a), Some(b)], [None, _]) => Ok((
                    [Some(a.into()), Some(b.into())],
                    [Some(Gate::Or.apply(&[a, b]).unwrap().into()), None],
                )),
                ([Some(a), Some(b)], [Some(v), _])
                    if Gate::Or.check(&[a, b], &[v]).is_ok_and(|x| x) =>
                {
                    Ok(([Some(a.into()), Some(b.into())], [Some(v.into()), None]))
                }
                _ => ARes::Err(anyhow!("Incorrect assingment for {self}")),
            },
            Self::Nor => {
                let (ins, outs) = Gate::And.set_value(
                    assingment_input.map(|x| x.map(|x| x.inverse())),
                    assignment_output,
                )?;
                let mut vals = [None, None];
                for i in 0..2 {
                    if let Some(value) = assingment_input[i] {
                        vals[i] = Some(BitString::from(value));
                    } else {
                        vals[i] = ins[i].map(BitString::flip);
                    }
                }
                Ok((vals, outs))
            }
            Self::Nand => {
                let (ins, outs) = Gate::Or.set_value(
                    assingment_input.map(|x| x.map(|x| x.inverse())),
                    assignment_output,
                )?;
                let mut vals = [None, None];
                for i in 0..2 {
                    if let Some(value) = assingment_input[i] {
                        vals[i] = Some(BitString::from(value));
                    } else {
                        vals[i] = ins[i].map(BitString::flip);
                    }
                }
                Ok((vals, outs))
            }
            Gate::Purify => match (assingment_input, assignment_output) {
                // Inputs
                ([Some(b), _], [None, None]) if b != b.inverse() => {
                    Ok(([Some(b.into()), None], [Some(b.into()), Some(b.into())]))
                }
                ([Some(b), _], [None, None]) if b == b.inverse() => Ok((
                    [Some(b.into()), None],
                    [
                        Some(BitString::smaller_than(Value::Bot)),
                        Some(BitString::greater_than(Value::Bot)),
                    ],
                )),
                // First bit values
                ([None, _], [Some(Value::Bot), None])
                | ([Some(Value::Bot), None], [Some(Value::Bot), None]) => Ok((
                    [Some(BitString::from(Value::Bot)), None],
                    [Some(Value::Bot.into()), Some(Value::One.into())],
                )),
                ([None, _], [Some(Value::One), None])
                | ([Some(Value::One), None], [Some(Value::One), None]) => Ok((
                    [Some(Value::One.into()), None],
                    [Some(Value::One.into()), Some(Value::One.into())],
                )),
                ([None, _], [Some(Value::Zero), None]) => Ok((
                    [Some(BitString::smaller_than(Value::Bot)), None],
                    [Some(Value::Zero.into()), Some(BitString::all())],
                )),
                // Second bit values
                ([None, _], [None, Some(Value::Bot)])
                | ([Some(Value::Bot), None], [None, Some(Value::Bot)]) => Ok((
                    [Some(Value::Bot.into()), None],
                    [Some(Value::Zero.into()), Some(Value::Bot.into())],
                )),
                ([None, _], [None, Some(Value::One)]) => Ok((
                    [Some(BitString::greater_than(Value::Bot)), None],
                    [Some(BitString::all()), Some(Value::One.into())],
                )),
                ([None, _], [None, Some(Value::Zero)])
                | ([Some(Value::Zero), _], [None, Some(Value::Zero)]) => Ok((
                    [Some(Value::Zero.into()), None],
                    [Some(Value::Zero.into()), Some(Value::Zero.into())],
                )),
                // Double outs
                ([None, _], [Some(a), Some(b)])
                    if VoltageOrdering(a) < VoltageOrdering(b) || (a == b && a != Value::Bot) =>
                {
                    Ok((
                        [
                            Some(if a == b { a.into() } else { Value::Bot.into() }),
                            None,
                        ],
                        [Some(a.into()), Some(b.into())],
                    ))
                }

                // Out in
                ([Some(a), _], [Some(b), None]) | ([Some(a), None], [None, Some(b)])
                    if a == b && a != Value::Bot =>
                {
                    Ok(([Some(a.into()), None], [Some(a.into()), Some(a.into())]))
                }
                ([Some(Value::Bot), _], [Some(b), None]) if b != Value::One => Ok((
                    [Some(Value::Bot.into()), None],
                    [
                        Some(b.into()),
                        Some(BitString::greater_than(Value::Bot).remove(b)),
                    ],
                )),
                ([Some(Value::Bot), _], [None, Some(b)]) if b != Value::Zero => Ok((
                    [Some(Value::Bot.into()), None],
                    [
                        Some(BitString::smaller_than(Value::Bot).remove(b)),
                        Some(b.into()),
                    ],
                )),
                // All
                ([Some(a), _], [Some(b), Some(c)])
                    if Gate::Purify.check(&[a], &[b, c]).is_ok_and(|x| x) =>
                {
                    Ok(([Some(a.into()), None], [Some(b.into()), Some(c.into())]))
                }
                _ => ARes::Err(anyhow!("Incorrect assingment for {self}")),
            },
            #[allow(unreachable_patterns)]
            _ => unimplemented!("Gate {self} was not implemented"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod bitset {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn check_range(s in any::<u8>()) {
                let b = BitString::try_from(s);
                if s >= 8 {
                    prop_assert!(b.is_err());
                } else {
                    prop_assert!(b.is_ok());
                }
            }

            #[test]
            fn check_valid(s in 0u8..8) {
                let b = BitString::try_from(s).unwrap();
                prop_assert_eq!((b.0) as u8, s & 1);
                prop_assert_eq!((b.1) as u8, (s & 2) >> 1);
                prop_assert_eq!((b.2) as u8, (s & 4) >> 2);
            }
        }

        #[test]
        fn test_operations() {
            assert_eq!(
                BitString::all(),
                BitString::from(Value::One).insert(Value::Zero).insert(Value::Bot)
            );

            assert_eq!(
                BitString::from(Value::One),
                BitString::from(Value::One).insert(Value::One).insert(Value::One)
            );

            assert_eq!(
                BitString::all().remove(Value::Bot),
                BitString::from(Value::One).insert(Value::Zero)
            );
            assert_eq!(
                BitString::all()
                    .remove(Value::Bot)
                    .op_inter(BitString::all().remove(Value::One)),
                BitString::from(Value::Zero),
            );
            assert_eq!(
                BitString::default()
                    .insert(Value::One)
                    .op_union(BitString::default().insert(Value::Zero)),
                BitString::all().remove(Value::Bot),
            )
        }
    }

    mod gate_evals {
        use crate::test_utils::enum_strategy;

        use rstest::rstest;

        use super::*;
        use proptest::prelude::*;

        fn other_value_generator() -> impl Strategy<Value = Option<Value>> {
            prop_oneof![Just(None), enum_strategy::<Value>().prop_map(Some)]
        }
        mod uniary {
            use super::*;

            proptest! {

                #[test]
                fn preserves_unary(a in other_value_generator(), b in other_value_generator(), gate in prop_oneof![
                    Just(Gate::Not),
                    Just(Gate::Copy),
                ]){

                    let res = gate.set_value([a, None], [b, None]);
                    prop_assume!(res.is_ok());
                    let (ins, outs) = res.unwrap();
                    if a.is_some() {
                        prop_assert_eq!(ins[0], a.map(Value::into));
                    }
                    if b.is_some() {
                        prop_assert_eq!(outs[0], b.map(Value::into));
                    }
                }
            }
            proptest! {
                #[test]
                fn test_copy(s in enum_strategy::<Value>(), other_1 in other_value_generator(), other_2 in other_value_generator()) {
                    let val = Gate::Copy.set_value([Some(s), other_1], [None, other_2]).unwrap();
                    let expect  = ([Some(BitString::from(s)), None], [Some(BitString::from(s)), None]);
                    prop_assert_eq!(val, expect);

                    let val = Gate::Copy.set_value([None, other_1], [Some(s), other_2]).unwrap();
                    let expect  = ([Some(BitString::from(s)), None], [Some(BitString::from(s)), None]);
                    prop_assert_eq!(val, expect);
                }

                #[test]
                fn test_not(s in enum_strategy::<Value>(), other_1 in other_value_generator(), other_2 in other_value_generator()) {
                    let val = Gate::Not.set_value([Some(s), other_1], [None, other_2]).unwrap();
                    let expect  = ([Some(BitString::from(s)), None], [Some(BitString::from(s.inverse())), None]);
                    prop_assert_eq!(val, expect);

                    let val = Gate::Not.set_value([None, other_1], [Some(s), other_2]).unwrap();
                    let expect  = ([Some(BitString::from(s.inverse())), None], [Some(BitString::from(s)), None]);
                    prop_assert_eq!(val, expect);
                }
            }
        }

        mod binary {
            use super::*;
            fn shift<T: Clone>(mut arr: [T; MAX_DEGREE], s: usize) -> [T; MAX_DEGREE] {
                let s = s.rem_euclid(arr.len());
                arr.rotate_left(s);
                arr
            }

            mod and {

                use super::*;

                #[rstest]
                fn test_and_single_assingment_out(
                    #[values(Value::Zero, Value::Bot, Value::One)] value: Value,
                ) {
                    let vals = Gate::And
                        .set_value([None, None], [Some(value), None])
                        .unwrap();
                    let expected = (
                        [
                            Some(BitString::greater_than(value)),
                            Some(BitString::greater_than(value)),
                        ],
                        [Some(value.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(0)]
                #[case(1)]
                fn test_and_single_assingment_inp(
                    #[values(Value::Zero, Value::Bot, Value::One)] s1: Value,
                    #[case] index: usize,
                ) {
                    let vals = Gate::And
                        .set_value(shift([None, Some(s1)], index), [None, None])
                        .unwrap();
                    let expected = (
                        shift([Some(BitString::all()), Some(BitString::from(s1))], index),
                        [Some(BitString::smaller_than(s1)), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(Value::One, Value::Zero)]
                #[case(Value::Bot, Value::Zero)]
                #[case(Value::One, Value::Bot)]
                fn test_and_double_assingment_and(
                    #[case] s1: Value,
                    #[case] s2: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::And
                        .set_value(shift([None, Some(s1)], index), [Some(s2), None])
                        .unwrap();
                    let expected = (
                        shift([Some(s2.into()), Some(s1.into())], index),
                        [Some(s2.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                fn test_and_double_assingment_and_equal(
                    #[values(Value::Zero, Value::Bot, Value::One)] s1: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::And
                        .set_value(shift([None, Some(s1)], index), [Some(s1), None])
                        .unwrap();
                    let expected = (
                        shift(
                            [Some(BitString::greater_than(s1.into())), Some(s1.into())],
                            index,
                        ),
                        [Some(s1.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(Value::Bot, Value::One)]
                #[case(Value::Zero, Value::One)]
                #[case(Value::Zero, Value::Bot)]
                fn test_and_double_assingment_and_fail(
                    #[case] s1: Value,
                    #[case] s2: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals =
                        Gate::And.set_value(shift([None, Some(s1)], index), [Some(s2), None]);

                    assert!(vals.is_err());
                }
            }

            #[rstest]
            fn test_and_double_assingment_and_on_ins(
                #[values(Value::Zero, Value::Bot, Value::One)] s1: Value,
                #[values(Value::Zero, Value::Bot, Value::One)] s2: Value,
                #[values(Gate::Nor, Gate::Nand, Gate::And, Gate::Or)] gate: Gate,
            ) {
                let vals = gate.set_value([Some(s1), Some(s2)], [None, None]).unwrap();
                let expected = (
                    [Some(s1.into()), Some(s2.into())],
                    [Some(gate.apply(&[s1, s2]).unwrap().into()), None],
                );

                assert_eq!(vals, expected);
            }

            #[rstest]
            fn test_and_triple_assingment(
                #[values(Value::Zero, Value::Bot, Value::One)] s1: Value,
                #[values(Value::Zero, Value::Bot, Value::One)] s2: Value,
                #[values(Value::Zero, Value::Bot, Value::One)] s3: Value,
                #[values(Gate::Nor, Gate::Nand, Gate::And, Gate::Or)] gate: Gate,
            ) {
                let vals = gate.set_value([Some(s1), Some(s2)], [Some(s3), None]);
                let expected = ([Some(s1.into()), Some(s2.into())], [Some(s3.into()), None]);
                if gate.apply(&[s1, s2]).unwrap() == s3 {
                    assert_eq!(vals.unwrap(), expected);
                } else {
                    assert!(vals.is_err(), "{vals:?}");
                }
            }

            mod or {
                use super::*;

                #[rstest]
                fn test_or_single_assingment_out(
                    #[values(Value::Zero, Value::Bot, Value::One)] s: Value,
                ) {
                    let vals = Gate::Or.set_value([None, None], [Some(s), None]).unwrap();
                    let expected = (
                        [
                            Some(BitString::smaller_than(s)),
                            Some(BitString::smaller_than(s)),
                        ],
                        [Some(s.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                fn test_and_single_assingment_inp(
                    #[values(0, 1)] index: usize,
                    #[values(Value::Zero, Value::Bot, Value::One)] starting: Value,
                ) {
                    let vals = Gate::Or
                        .set_value(shift([None, Some(starting)], index), [None, None])
                        .unwrap();
                    let expected = (
                        shift([Some(BitString::all()), Some(starting.into())], index),
                        [Some(BitString::greater_than(starting)), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(Value::Zero, Value::Bot)]
                #[case(Value::Zero, Value::One)]
                #[case(Value::Bot, Value::One)]
                fn test_and_double_assingment_or(
                    #[case] s1: Value,
                    #[case] s2: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::Or
                        .set_value(shift([None, Some(s1)], index), [Some(s2), None])
                        .unwrap();
                    let expected = (
                        shift([Some(s2.into()), Some(s1.into())], index),
                        [Some(s2.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                fn test_and_double_assingment_and_equal(
                    #[values(Value::Zero, Value::Bot, Value::One)] s1: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::Or
                        .set_value(shift([None, Some(s1)], index), [Some(s1), None])
                        .unwrap();
                    let expected = (
                        shift(
                            [Some(BitString::smaller_than(s1.into())), Some(s1.into())],
                            index,
                        ),
                        [Some(s1.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(Value::Bot, Value::Zero)]
                #[case(Value::One, Value::Zero)]
                #[case(Value::One, Value::Bot)]
                fn test_and_double_assingment_and_fail(
                    #[case] s1: Value,
                    #[case] s2: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::Or.set_value(shift([None, Some(s1)], index), [Some(s2), None]);

                    assert!(vals.is_err());
                }
            }

            mod nor {
                use super::*;

                #[rstest]
                fn test_nor_single_assingment_out(
                    #[values(Value::Zero, Value::Bot, Value::One)] s: Value,
                ) {
                    let vals = Gate::Nor.set_value([None, None], [Some(s), None]).unwrap();
                    let expected = (
                        [
                            Some(BitString::smaller_than(s.inverse())),
                            Some(BitString::smaller_than(s.inverse())),
                        ],
                        [Some(s.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                fn test_nor_single_assingment_inp(
                    #[values(0, 1)] index: usize,
                    #[values(Value::Zero, Value::Bot, Value::One)] starting: Value,
                ) {
                    let vals = Gate::Nor
                        .set_value(shift([None, Some(starting)], index), [None, None])
                        .unwrap();
                    let expected = (
                        shift([Some(BitString::all()), Some(starting.into())], index),
                        [Some(BitString::smaller_than(starting.inverse())), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(Value::Zero, Value::Bot)]
                #[case(Value::Zero, Value::One)]
                #[case(Value::Bot, Value::Zero)]
                fn test_nor_double_assingment_or(
                    #[case] s1: Value,
                    #[case] s2: Value,

                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::Nor
                        .set_value(shift([None, Some(s1)], index), [Some(s2), None])
                        .unwrap();
                    let expected = (
                        shift([Some(s2.inverse().into()), Some(s1.into())], index),
                        [Some(s2.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]

                fn test_nor_double_assingment_and_equal(
                    #[values(Value::Zero, Value::Bot, Value::One)] s1: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::Nor
                        .set_value(shift([None, Some(s1)], index), [Some(s1.inverse()), None])
                        .unwrap();
                    let expected = (
                        shift(
                            [Some(BitString::smaller_than(s1.into())), Some(s1.into())],
                            index,
                        ),
                        [Some(s1.inverse().into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(Value::Bot, Value::One)]
                #[case(Value::One, Value::One)]
                #[case(Value::One, Value::Bot)]
                // #[case(Value::Zero, Value::Bot)]
                fn test_nor_double_assingment_and_fail(
                    #[case] s1: Value,
                    #[case] s2: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals =
                        Gate::Nor.set_value(shift([None, Some(s1)], index), [Some(s2), None]);

                    assert!(vals.is_err());
                }
            }

            mod nand {
                use super::*;

                #[rstest]
                fn test_nand_single_assingment_out(
                    #[values(Value::Zero, Value::Bot, Value::One)] s: Value,
                ) {
                    let vals = Gate::Nand.set_value([None, None], [Some(s), None]).unwrap();
                    let expected = (
                        [
                            Some(BitString::greater_than(s.inverse())),
                            Some(BitString::greater_than(s.inverse())),
                        ],
                        [Some(s.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                fn test_nand_single_assingment_inp(
                    #[values(0, 1)] index: usize,
                    #[values(Value::Zero, Value::Bot, Value::One)] starting: Value,
                ) {
                    let vals = Gate::Nand
                        .set_value(shift([None, Some(starting)], index), [None, None])
                        .unwrap();
                    let expected = (
                        shift([Some(BitString::all()), Some(starting.into())], index),
                        [Some(BitString::greater_than(starting.inverse())), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(Value::One, Value::One)]
                #[case(Value::One, Value::Bot)]
                #[case(Value::Bot, Value::One)]
                fn test_nand_double_assingment_or(
                    #[case] s1: Value,
                    #[case] s2: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::Nand
                        .set_value(shift([None, Some(s1)], index), [Some(s2), None])
                        .unwrap();
                    let expected = (
                        shift([Some(s2.inverse().into()), Some(s1.into())], index),
                        [Some(s2.into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]

                fn test_nand_double_assingment_and_equal(
                    #[values(Value::Zero, Value::Bot, Value::One)] s1: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals = Gate::Nand
                        .set_value(shift([None, Some(s1)], index), [Some(s1.inverse()), None])
                        .unwrap();
                    let expected = (
                        shift(
                            [Some(BitString::greater_than(s1.into())), Some(s1.into())],
                            index,
                        ),
                        [Some(s1.inverse().into()), None],
                    );

                    assert_eq!(vals, expected);
                }

                #[rstest]
                #[case(Value::Zero, Value::Bot)]
                #[case(Value::Zero, Value::Zero)]
                #[case(Value::Bot, Value::Zero)]
                fn test_nand_double_assingment_and_fail(
                    #[case] s1: Value,
                    #[case] s2: Value,
                    #[values(0, 1)] index: usize,
                ) {
                    let vals =
                        Gate::Nand.set_value(shift([None, Some(s1)], index), [Some(s2), None]);

                    assert!(vals.is_err());
                }
            }

            proptest! {

                #[test]
                fn preserves_binary(a in other_value_generator(), b in other_value_generator(), c in other_value_generator(), gate in prop_oneof![
                    Just(Gate::And),
                    Just(Gate::Or),
                    Just(Gate::Nand),
                    Just(Gate::Nor),
                ]){

                    let res = gate.set_value([a, b], [c, None]);
                    prop_assume!(res.is_ok());
                    let (ins, outs) = res.unwrap();
                    if a.is_some() {
                        prop_assert_eq!(ins[0], a.map(Value::into));
                    }
                    if b.is_some() {
                        prop_assert_eq!(ins[1], b.map(Value::into));
                    }
                    if c.is_some() {
                        prop_assert_eq!(outs[0], c.map(Value::into));
                    }

                }
            }
        }

        mod purify {
            use super::*;
            #[rstest]
            fn single_input_pure(#[values(Value::Zero, Value::One)] inp: Value) {
                let vals = Gate::Purify
                    .set_value([Some(inp), None], [None, None])
                    .unwrap();
                let expected = (
                    [Some(inp.into()), None],
                    [Some(inp.into()), Some(inp.into())],
                );

                assert_eq!(vals, expected);
            }

            #[test]
            fn single_input_bot() {
                let vals = Gate::Purify
                    .set_value([Some(Value::Bot), None], [None, None])
                    .unwrap();
                let expected = (
                    [Some(Value::Bot.into()), None],
                    [
                        Some(BitString::smaller_than(Value::Bot)),
                        Some(BitString::greater_than(Value::Bot)),
                    ],
                );

                assert_eq!(vals, expected);
            }

            #[test]
            fn output_left_purify() {
                let vals = Gate::Purify
                    .set_value([None, None], [Some(Value::Zero), None])
                    .unwrap();
                let expected = (
                    [Some(BitString::smaller_than(Value::Bot)), None],
                    [Some(BitString::from(Value::Zero)), Some(BitString::all())],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([None, None], [Some(Value::Bot), None])
                    .unwrap();
                let expected = (
                    [Some(Value::Bot.into()), None],
                    [Some(Value::Bot.into()), Some(Value::One.into())],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([None, None], [Some(Value::One), None])
                    .unwrap();
                let expected = (
                    [Some(Value::One.into()), None],
                    [Some(Value::One.into()), Some(Value::One.into())],
                );

                assert_eq!(vals, expected);
            }

            #[test]
            fn output_right_purify() {
                let vals = Gate::Purify
                    .set_value([None, None], [None, Some(Value::Zero)])
                    .unwrap();
                let expected = (
                    [Some(Value::Zero.into()), None],
                    [Some(Value::Zero.into()), Some(Value::Zero.into())],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([None, None], [None, Some(Value::Bot)])
                    .unwrap();
                let expected = (
                    [Some(Value::Bot.into()), None],
                    [Some(Value::Zero.into()), Some(Value::Bot.into())],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([None, None], [None, Some(Value::One)])
                    .unwrap();
                let expected = (
                    [Some(BitString::greater_than(Value::Bot)), None],
                    [Some(BitString::all()), Some(Value::One.into())],
                );

                assert_eq!(vals, expected);
            }

            #[rstest]
            fn double_out_valid_pure(#[values(Value::Zero, Value::One)] val: Value) {
                let vals = Gate::Purify
                    .set_value([None, None], [Some(val), Some(val)])
                    .unwrap();
                let expected = (
                    [Some(val.into()), None],
                    [Some(val.into()), Some(val.into())],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([Some(val), None], [None, Some(val)])
                    .unwrap();
                let expected = (
                    [Some(val.into()), None],
                    [Some(val.into()), Some(val.into())],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([Some(val), None], [Some(val), None])
                    .unwrap();
                let expected = (
                    [Some(val.into()), None],
                    [Some(val.into()), Some(val.into())],
                );

                assert_eq!(vals, expected);
            }

            #[rstest]
            #[case(Value::Bot, Value::Bot)]
            #[case(Value::One, Value::Bot)]
            #[case(Value::Bot, Value::Zero)]
            #[case(Value::One, Value::Zero)]
            #[should_panic]
            fn double_out_valid_pure_should_fail(#[case] a: Value, #[case] b: Value) {
                let _ = Gate::Purify
                    .set_value([None, None], [Some(a), Some(b)])
                    .unwrap();
            }

            #[rstest]
            #[case(Value::Bot, Value::One)]
            #[case(Value::Zero, Value::Bot)]
            #[case(Value::Zero, Value::One)]
            fn double_out_valid_bot_should_pass(#[case] a: Value, #[case] b: Value) {
                let vals = Gate::Purify
                    .set_value([None, None], [Some(a), Some(b)])
                    .unwrap();
                let expected = (
                    [Some(Value::Bot.into()), None],
                    [Some(a.into()), Some(b.into())],
                );

                assert_eq!(vals, expected);
            }

            #[test]
            fn double_in_out_valid_bot_should_pass() {
                let vals = Gate::Purify
                    .set_value([Some(Value::Bot), None], [Some(Value::Bot), None])
                    .unwrap();
                let expected = (
                    [Some(Value::Bot.into()), None],
                    [Some(Value::Bot.into()), Some(Value::One.into())],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([Some(Value::Bot), None], [Some(Value::Zero), None])
                    .unwrap();
                let expected = (
                    [Some(Value::Bot.into()), None],
                    [
                        Some(Value::Zero.into()),
                        Some(BitString::greater_than(Value::Bot)),
                    ],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([Some(Value::Bot), None], [None, Some(Value::Bot)])
                    .unwrap();
                let expected = (
                    [Some(Value::Bot.into()), None],
                    [Some(Value::Zero.into()), Some(Value::Bot.into())],
                );

                assert_eq!(vals, expected);

                let vals = Gate::Purify
                    .set_value([Some(Value::Bot), None], [None, Some(Value::One)])
                    .unwrap();
                let expected = (
                    [Some(Value::Bot.into()), None],
                    [
                        Some(BitString::smaller_than(Value::Bot)),
                        Some(Value::One.into()),
                    ],
                );

                assert_eq!(vals, expected);
            }

            #[rstest]
            #[case(Some(Value::Zero), Some(Value::One), None)]
            #[case(Some(Value::Zero), Some(Value::Bot), None)]
            #[case(Some(Value::Zero), None, Some(Value::One))]
            #[case(Some(Value::Zero), None, Some(Value::Bot))]
            #[case(Some(Value::One), Some(Value::Zero), None)]
            #[case(Some(Value::One), Some(Value::Bot), None)]
            #[case(Some(Value::One), None, Some(Value::Zero))]
            #[case(Some(Value::One), None, Some(Value::Bot))]
            #[case(Some(Value::Bot), None, Some(Value::Zero))]
            #[case(Some(Value::Bot), Some(Value::One), None)]
            #[should_panic]
            fn double_in_out_valid_bot_should_fail(
                #[case] in1: Option<Value>,
                #[case] out1: Option<Value>,
                #[case] out2: Option<Value>,
            ) {
                let _ = Gate::Purify.set_value([in1, None], [out1, out2]).unwrap();
            }

            #[rstest]
            fn all_triplets_value(
                #[values(Value::Zero, Value::Bot, Value::One)] a: Value,
                #[values(Value::Zero, Value::Bot, Value::One)] b: Value,
                #[values(Value::Zero, Value::Bot, Value::One)] c: Value,
            ) {
                let res = Gate::Purify.set_value([Some(a), None], [Some(b), Some(c)]);
                match (res, Gate::Purify.check(&[a], &[b, c]).is_ok_and(|x| x)) {
                    (res, false) => assert!(res.is_err(), "{res:?}"),
                    (res, true) => {
                        assert!(res.is_ok());
                        let res = res.unwrap();
                        assert_eq!(
                            ([Some(a.into()), None], [Some(b.into()), Some(c.into())]),
                            res
                        );
                    }
                }
            }

            proptest! {

                #[test]
                fn preserves_purify(a in other_value_generator(), b in other_value_generator(), c in other_value_generator()){

                    let res = Gate::Purify.set_value([a, None], [b,c]);
                    prop_assume!(res.is_ok());
                    let (ins, outs) = res.unwrap();
                    if a.is_some() {
                        prop_assert_eq!(ins[0], a.map(Value::into));
                    }
                    if b.is_some() {
                        prop_assert_eq!(outs[0], b.map(Value::into));
                    }
                    if c.is_some() {
                        prop_assert_eq!(outs[1], c.map(Value::into));
                    }

                }
            }
        }
    }
}
