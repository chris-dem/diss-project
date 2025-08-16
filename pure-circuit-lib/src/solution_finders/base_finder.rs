use crate::{
    gates::{Gate, NewNode, NodeUnitialised, NodeValue, Value},
    graph::PureCircuitGraph,
};
use genetic_algorithm::allele::Allele;
use itertools::{EitherOrBoth, Itertools};
use petgraph::{
    prelude::*,
    unionfind::UnionFind,
    visit::{EdgeRef, IntoEdgeReferences, NodeIndexable},
};
use std::collections::HashMap;
use std::fmt::Debug;

pub const MAX_DEGREE: usize = 2;
type Inner = (
    Gate,
    [Option<usize>; MAX_DEGREE],
    [Option<usize>; MAX_DEGREE],
);

#[derive(Debug, Clone, Default)]
pub struct FitnessPureCircuit(Box<[Inner]>);

impl Allele for Value {}
impl<T: Debug + Copy, G: Debug + Copy> PureCircuitGraph<T, G> {
    pub fn to_chromosone(&self) -> Box<[Value]> {
        self.graph
            .node_weights()
            .filter_map(|n| match n.into_node() {
                NodeValue::ValueNode(value) => Some(value),
                NodeValue::GateNode { .. } => None,
            })
            .collect()
    }

    pub fn from_chromosone(&mut self, chromosome: &[Value]) -> Option<()> {
        for e in self
            .graph
            .node_indices()
            .filter(|n| !self.graph[*n].node.is_gate())
            .zip_longest(chromosome.into_iter())
            .collect::<Box<[_]>>()
        {
            match e {
                EitherOrBoth::Both(indx, value) => {
                    if let Err(e) = self.update_node(indx, NodeUnitialised::from_value(*value)) {
                        dbg!(e);
                        return None;
                    }
                }
                _ => return None,
            };
        }
        Some(())
    }

    // fn get_conn_components(&self) -> Vec<usize> {
    //     let mut node_sets = UnionFind::new(self.graph.node_bound());
    //     for edge in self.graph.edge_references() {
    //         let (a, b) = (edge.source(), edge.target());

    //         // union the two nodes of the edge
    //         node_sets.union(self.graph.to_index(a), self.graph.to_index(b));
    //     }

    //     node_sets.into_labeling()
    // }

    pub fn to_fitness_function(&self) -> Option<FitnessPureCircuit> {
        let map = self
            .graph
            .node_indices()
            .filter_map(|i| match self.graph[i].into_node() {
                NodeValue::ValueNode(_) => Some(i),
                NodeValue::GateNode { .. } => None,
            })
            .enumerate()
            .map(|(a, b)| (b, a))
            .collect::<HashMap<NodeIndex, usize>>();

        let mapper = self
            .graph
            .node_indices()
            .filter_map(|i| match self.graph[i].into_node() {
                NodeValue::ValueNode(_) => None,
                NodeValue::GateNode { gate, .. } => Some((i, gate)),
            })
            .map(|(nod_ind, gate)| {
                let mut ret_in = [None; MAX_DEGREE];
                let mut ret_out = [None; MAX_DEGREE];
                for (node, ind) in self
                    .graph
                    .neighbors_directed(nod_ind, Direction::Incoming)
                    .collect_vec()
                    .into_iter()
                    .rev()
                    .enumerate()
                {
                    ret_in[node] = Some(ind)
                }

                for (node, ind) in self
                    .graph
                    .neighbors_directed(nod_ind, Direction::Outgoing)
                    .collect_vec()
                    .into_iter()
                    .rev()
                    .enumerate()
                {
                    ret_out[node] = Some(ind)
                }

                (gate, ret_in, ret_out)
            })
            .map::<Option<Inner>, _>(|(g, in_ind, out_ind)| {
                if g.arity()
                    == (
                        in_ind.iter().copied().filter(Option::is_some).count(),
                        out_ind.iter().copied().filter(Option::is_some).count(),
                    )
                {
                    let in_ind = in_ind
                        .iter()
                        .copied()
                        .map(|e| e.map(|e| *map.get(&e).unwrap()))
                        .collect_array::<MAX_DEGREE>()?;
                    let out_ind = out_ind
                        .iter()
                        .copied()
                        .map(|e| e.map(|e| *map.get(&e).unwrap()))
                        .collect_array::<MAX_DEGREE>()?;
                    Some((g, in_ind, out_ind))
                } else {
                    None
                }
            })
            .collect::<Option<Box<[Inner]>>>()?;
        Some(FitnessPureCircuit(mapper))
    }
}

impl FitnessPureCircuit {
    // TODO FIX
    pub fn evaluate(&self, inputs: &[Value]) -> Option<usize> {
        todo!("Not good enough since ordering is maybe lost. Verify");
        let mut errors = 0usize;
        let t = inputs;
        for (g, ins, outs) in self.0.iter().copied() {
            let ins = ins
                .into_iter()
                .filter_map(|s| s.map(|ind| t[ind]))
                .collect_vec();
            let outs = outs
                .into_iter()
                .filter_map(|s| s.map(|ind| t[ind]))
                .collect_vec();
            match g.check(ins.as_slice(), outs.as_slice()) {
                Err(e) => {
                    dbg!(e);
                    return None;
                }
                Ok(b) => errors += !b as usize,
            }
        }

        Some(errors)
    }
}

#[cfg(test)]
mod test_evo {

    use super::*;

    use crate::{gates::NodeUnitialised, test_utils::enum_strategy};
    use proptest::prelude::{Strategy, *};

    // #[derive(Debug, Clone)]
    // struct PurifyFitness;

    mod conversion_tests {
        use super::*;

        fn generate_strategy() -> impl proptest::prelude::Strategy<Value = Vec<NodeUnitialised>> {
            proptest::collection::vec(
                prop_oneof![
                    enum_strategy::<Value>().prop_map(NodeUnitialised::from_value),
                    enum_strategy::<Gate>().prop_map(NodeUnitialised::from_gate)
                ],
                10..=250,
            )
        }

        proptest! {

            #[test]
            fn to_chromosone_alignment(s in generate_strategy()) {
                let mut pc = PureCircuitGraph::<(), ()>::new();
                for el in s.iter().copied() {
                    pc.add_node(el, ());
                }
                let el1 = pc.to_chromosone();
                let el2 = s.into_iter().filter_map(|e| if let NodeValue::ValueNode(v) = e {
                    Some(v)
                } else { None }).collect::<Box<[_]>>();
                prop_assert_eq!(el1, el2);

            }
        }

        #[test]
        fn check_mapping_1() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            pc.add_edge(val_2, gate_1, ()).unwrap();
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_3, ()).unwrap();
            let fitness = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fitness.0,
                *Box::new([(Gate::And, [Some(1), Some(0)], [Some(2), None])])
            );
        }

        #[test]
        fn check_mapping_2() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(val_2, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_3, ()).unwrap();
            let fitness = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fitness.0,
                *Box::new([(Gate::And, [Some(0), Some(1)], [Some(2), None])])
            );
        }

        #[test]
        fn check_mapping_3() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(val_3, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_2, ()).unwrap();
            let fitness = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fitness.0,
                *Box::new([(Gate::And, [Some(0), Some(2)], [Some(1), None])])
            );
        }

        #[test]
        fn check_mapping_4() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_2, ()).unwrap();
            assert!(matches!(pc.to_fitness_function(), None));
        }

        #[test]
        fn check_mapping_big_1() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_4 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let gate_2 = pc.add_node(NodeUnitialised::from_gate(Gate::Copy), ());
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(val_2, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_3, ()).unwrap();
            pc.add_edge(val_3, gate_2, ()).unwrap();
            pc.add_edge(gate_2, val_4, ()).unwrap();
            let fit = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fit.0,
                *Box::new([
                    (Gate::And, [Some(0), Some(1)], [Some(2), None]),
                    (Gate::Copy, [Some(2), None], [Some(3), None]),
                ])
            );
        }

        #[test]
        fn check_mapping_big_2() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ()); // 0
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ()); // 1
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_4 = pc.add_node(NodeUnitialised::from_value(Value::One), ()); // 2
            let gate_2 = pc.add_node(NodeUnitialised::from_gate(Gate::Copy), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ()); //3
            pc.add_edge(gate_2, val_4, ()).unwrap();
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_3, ()).unwrap();
            pc.add_edge(val_2, gate_1, ()).unwrap();
            pc.add_edge(val_3, gate_2, ()).unwrap();
            let fit = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fit.0,
                *Box::new([
                    (Gate::And, [Some(0), Some(3)], [Some(1), None]),
                    (Gate::Copy, [Some(1), None], [Some(2), None]),
                ])
            );
        }
    }

    mod fitness_tests {
        use super::*;

        proptest! {

                #[test]
                fn check_simple_case(x in enum_strategy::<Value>(), y in enum_strategy::<Value>(), z in enum_strategy::<Value>()) {
                    let mut pc = PureCircuitGraph::<(), ()>::new();
                    let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
                    let val_1 = pc.add_node(NodeUnitialised::from_value(x), ());
                    let val_2 = pc.add_node(NodeUnitialised::from_value(y), ());
                    let val_3 = pc.add_node(NodeUnitialised::from_value(z), ());
                    pc.add_edge(val_2, gate_1, ()).unwrap();
                    pc.add_edge(val_1, gate_1, ()).unwrap();
                    pc.add_edge(gate_1, val_3, ()).unwrap();
                    let fitness = pc.to_fitness_function().unwrap();

                    let s = fitness.evaluate(&vec![x,y,z]);
                    assert_eq!(s, Some(
                        (Gate::And.check(&[x,y], &[z]).unwrap() == false) as usize
                    ), "We are getting {}", Gate::And.check(&[x,y], &[z]).unwrap());
                }

                #[test]
                fn check_complex_case(s in prop::array::uniform5(enum_strategy::<Value>())) {
                    let mut pc = PureCircuitGraph::<(), ()>::new();
                    let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::Purify), ());
                    let val_1 = pc.add_node(NodeUnitialised::from_value(s[0]), ());
                    let val_2 = pc.add_node(NodeUnitialised::from_value(s[1]), ());
                    let gate_2 = pc.add_node(NodeUnitialised::from_gate(Gate::Or), ());
                    let val_3 = pc.add_node(NodeUnitialised::from_value(s[2]), ());
                    let val_4 = pc.add_node(NodeUnitialised::from_value(s[3]), ());
                    let val_5 = pc.add_node(NodeUnitialised::from_value(s[4]), ());
                    pc.add_edge(val_2, gate_1, ()).unwrap();
                    pc.add_edge(gate_1, val_1, ()).unwrap();
                    pc.add_edge(gate_1, val_3, ()).unwrap();
                    pc.add_edge(val_3, gate_2, ()).unwrap();
                    pc.add_edge(val_4, gate_2, ()).unwrap();
                    pc.add_edge(gate_2, val_5, ()).unwrap();
                    let fitness = pc.to_fitness_function().unwrap();
                    let res = fitness.evaluate(&s.to_vec());
                    assert_eq!(res, Some(
                        (Gate::Purify.check(&[s[1]], &[s[0], s[2]]).unwrap() == false) as usize
                        + (Gate::Or.check(&[s[2], s[3]], &[s[4]]).unwrap() == false) as usize
                    ));
            }
        }
    }
}
