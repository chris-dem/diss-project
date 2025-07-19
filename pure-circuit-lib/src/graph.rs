use itertools::Itertools;
use petgraph::prelude::*;

use crate::gates::{NewNode, NodeValue, Value};

#[derive(Debug, Clone)]
pub struct PureCircuitGraph(DiGraph<NodeValue, ()>);

impl Default for PureCircuitGraph {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub enum GraphNodeValue {
    Value(),
}

impl PureCircuitGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_error_gates(&self) -> impl Iterator<Item = (NodeIndex, NodeValue)> {
        self.0
            .node_weights()
            .enumerate()
            .filter_map(|(idx, node_val)| match node_val {
                NodeValue::GateNode { gate: _, status: _ } => {
                    Some((NodeIndex::new(idx), *node_val))
                }
                NodeValue::ValueNode(_) => None,
            })
    }

    pub fn add_node(&mut self, node: NewNode) -> NodeIndex {
        match node {
            NewNode::GateNode(g) => self.0.add_node(NodeValue::GateNode {
                gate: g,
                status: crate::gates::GateStatus::InvalidArity,
            }),
            NewNode::ValueNode(v) => self.0.add_node(NodeValue::ValueNode(v)),
        }
    }

    pub fn add_nodes(
        &mut self,
        nodes: impl Iterator<Item = NewNode>,
    ) -> impl Iterator<Item = NodeIndex> {
        nodes
            .map(|x| self.add_node(x))
            .collect::<Box<[NodeIndex]>>()
            .into_iter()
    }

    pub fn add_edge(&mut self, src_idx: NodeIndex, dest_idx: NodeIndex) -> Option<EdgeIndex> {
        todo!("Not implemented");
        // let src_node = self
        //     .graph
        //     .node_weight(src_idx)
        //     .ok_or(PureCircuitErrors::MissingNodes)?;
        // let dest_node = self
        //     .graph
        //     .node_weight(dest_idx)
        //     .ok_or(PureCircuitErrors::MissingNodes)?;

        // let ret = self.graph.add_edge(src_idx, dest_idx, ());
        // let incoming = self
        //     .graph
        //     .neighbors_directed(dest_idx, Direction::Incoming).collect::<Box<_>>();
        // let outgoing = self
        //     .graph
        //     .neighbors_directed(dest_idx, Direction::Outgoing).collect::<Box<_>>();
        // Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::Gate;
    use crate::gates::Value;
    use crate::test_utils::enum_strategy;
    use itertools::Itertools;
    use proptest::prelude::*;
    use proptest::strategy::BoxedStrategy;
    use std::fmt::Debug;
    use strum::IntoEnumIterator;

    pub fn sample_from_slice<T: Debug + Clone + 'static>(values: &[T]) -> BoxedStrategy<T> {
        prop::sample::select(values.to_vec()).boxed()
    }

    mod node_tests {

        use super::*;
        // proptest! {
        //     #[test]
        //     fn add_value_node(
        //         s in (prop::collection::linked_list(
        //         enum_strategy::<Value>()
        //             .prop_map(NewNode::ValueNode)
        //             // .prop_union(enum_strategy::<Gate>()
        //             //     .prop_map(NewNode::GateNode))

        //             // .chain(Gate::iter().map(NewNode::GateNode)).collect_vec()
        //         , 5..1_000))
        //     ){
        //         let mut pc = PureCircuitGraph::default();
        //         let mut counter = 0;
        //         for n in s {
        //             pc.add_node(n);
        //             if let NewNode::GateNode(_) = n {
        //                 counter += 1;
        //             }
        //         }
        //         let bad_nodes = pc.get_error_gates().count();

        //         prop_assert_eq!(counter, bad_nodes);
        //     }
        // }
    }

    mod edge_tests {
        use std::collections::HashSet;

        use proptest::test_runner::TestRng;

        use super::*;

        use crate::gates::{GateStatus, NewNode};

        fn generate_generic_graph() -> impl Strategy<Value = (HashSet<(usize, usize)>, Vec<NewNode>)>
        {
            prop::collection::vec(
                prop_oneof![
                    enum_strategy::<Value>().prop_map(NewNode::ValueNode),
                    enum_strategy::<Gate>().prop_map(NewNode::GateNode),
                ],
                0..=200,
            )
            .prop_flat_map(|arr| {
                (
                    prop::collection::hash_set((0..arr.len(), 0..arr.len()), 0..100),
                    Just(arr),
                )
            })
        }

        fn generate_heterogeneous_graph()
        -> impl Strategy<Value = (Box<[(bool, usize, usize)]>, Vec<NewNode>, Vec<NewNode>)>
        {
            (
                prop::collection::vec(
                    enum_strategy::<Value>().prop_map(NewNode::ValueNode),
                    0..=200,
                ),
                prop::collection::vec(enum_strategy::<Gate>().prop_map(NewNode::GateNode), 0..=200),
            )
                .prop_flat_map(|(arr_val, arr_gate)| {
                    (
                        (
                            prop::collection::btree_set(
                                (Just(true), 0..arr_val.len(), 0..arr_gate.len()),
                                0..100,
                            ),
                            prop::collection::btree_set(
                                (Just(false), 0..arr_gate.len(), 0..arr_val.len()),
                                0..100,
                            ),
                        )
                            .prop_flat_map(|(l, r)| {
                                Just(
                                    l.union(&r)
                                        .copied()
                                        .collect::<Box<[(bool, usize, usize)]>>(),
                                )
                            }),
                        Just(arr_val),
                        Just(arr_gate),
                    )
                })
        }

        proptest! {
            #[test]
            fn check_invalid_edges(s in generate_generic_graph()) {
                let mut graph = PureCircuitGraph::default();
                let (edges, array) = s;
                let node_indxes = graph.add_nodes(array.iter().cloned()).collect_vec();
                for (src, dst) in edges {
                    let flag = match (array[src], array[dst]) {
                        (NewNode::GateNode(_), NewNode::ValueNode(_))
                        | (NewNode::ValueNode(_), NewNode::GateNode(_)) => true,
                        _ => false,
                    };
                    prop_assert_eq!(
                        graph.add_edge(node_indxes[src], node_indxes[dst]).is_some(),
                        flag
                    );
                }
            }

            #[test]
            fn check_bad_nodes(s in generate_heterogeneous_graph()) {
                let mut graph = PureCircuitGraph::default();
                let (edges, vals, gates) = s;
                let vals_idx = graph.add_nodes(vals.into_iter()).collect_vec();
                let gates_idx = graph.add_nodes(gates.into_iter()).collect_vec();
                let mut checks = true;
                for (dir, src, dest) in edges {
                    if dir {
                        checks &= graph.add_edge(vals_idx[src], gates_idx[dest]).is_some();
                    } else {
                        checks &= graph.add_edge(gates_idx[src], vals_idx[dest]).is_some();
                    }
                }
                prop_assert!(checks);
                for node_ix in graph.0.node_indices() {
                    let node_weight = graph.0.node_weight(node_ix);
                    prop_assert!(node_weight.is_some());
                    let node_weight = node_weight.unwrap();
                    match node_weight {
                        NodeValue::ValueNode(_) => continue,
                        NodeValue::GateNode { gate, status } => {
                            let in_neigh = graph.0.neighbors_directed(node_ix,Direction::Incoming).map(|ind| graph.0.node_weight(ind).unwrap()).collect_vec();
                            let out_neigh = graph.0.neighbors_directed(node_ix,Direction::Outgoing).map(|ind| graph.0.node_weight(ind).unwrap()).collect_vec();
                            prop_assert!(in_neigh.iter().all(|f| matches!(f,NodeValue::ValueNode(_))));
                            prop_assert!(out_neigh.iter().all(|f| matches!(f,NodeValue::ValueNode(_))));
                            let in_neigh = in_neigh.into_iter().map(|el| {
                                let NodeValue::ValueNode(v) = el else { panic!("Should not happen")};
                                v
                            }).copied().collect_vec();
                            let out_neigh = out_neigh.into_iter().map(|el| {
                                let NodeValue::ValueNode(v) = el else { panic!("Should not happen")};
                                v
                            }).copied().collect_vec();
                            if (in_neigh.len(),out_neigh.len()) == gate.arity() {
                                prop_assert_ne!(*status, GateStatus::InvalidArity);
                                match gate.check(&in_neigh, &out_neigh) {
                                    Ok(b) => {
                                        prop_assert!(b == (*status == GateStatus::Valid))
                                    },
                                    Err(_) => prop_assert!(false, "Should not reach errored gate"),
                                }
                            } else {
                                prop_assert_eq!(*status, GateStatus::InvalidArity);
                            }
                        },
                    }
                }
            }


        }
    }
}
