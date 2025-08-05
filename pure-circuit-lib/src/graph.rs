use petgraph::{prelude::*, visit::IntoNeighborsDirected};
use strum_macros::Display;

use crate::gates::{GateError, GateStatus, GraphNode, NewNode, NodeUnitialised, NodeValue, Value};

#[derive(Debug, Clone)]
pub struct PureCircuitGraph {
    pub graph: DiGraph<GraphNode, u64>,
}

impl Default for PureCircuitGraph {
    fn default() -> Self {
        Self {
            graph: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum GraphError {
    NotExistentNode,
    NonHeterogeneousEdge,
    InvalidUpdate,
}

impl std::error::Error for GraphError {}

impl PureCircuitGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_edges(&self) -> impl Iterator<Item = (NodeIndex, NodeIndex)> {
        self.graph
            .edge_references()
            .map(|r| (r.source(), r.target()))
    }

    pub fn update_node(
        &mut self,
        index: NodeIndex,
        new_value: NodeUnitialised,
    ) -> Result<Box<[(NodeIndex, GateStatus)]>, GraphError> {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(GraphError::NotExistentNode)?;

        let copied_node = *node;
        match (node, new_value) {
            (NodeValue::GateNode { gate, .. }, NodeValue::GateNode { gate: g, .. }) => *gate = g,
            (NodeValue::ValueNode(val), NodeValue::ValueNode(e)) => *val = e,
            _ => return Err(GraphError::InvalidUpdate),
        };

        match copied_node {
            NodeValue::GateNode { .. } => {
                let gate = self.update_node_status(index).and_then(|g| {
                    if let NodeValue::GateNode { state_type, .. } = g {
                        Ok(state_type)
                    } else {
                        Err(GraphError::InvalidUpdate)
                    }
                })?;
                Ok(Box::new([(index, gate)]))
            }
            NodeValue::ValueNode(_) => {
                let mut ret = Vec::new();
                for n in self
                    .graph
                    .neighbors_directed(index, Direction::Incoming)
                    .chain(self.graph.neighbors_directed(index, Direction::Outgoing))
                    .collect::<Box<[NodeIndex]>>()
                {
                    let status = self.update_node_status(n).and_then(|g| {
                        if let NodeValue::GateNode { state_type, .. } = g {
                            Ok(state_type)
                        } else {
                            Err(GraphError::InvalidUpdate)
                        }
                    })?;
                    ret.push((n, status));
                }
                Ok(ret.iter().copied().collect::<_>())
            }
        }
    }

    pub fn get_error_gates(&self) -> impl Iterator<Item = (NodeIndex, GraphNode)> {
        self.graph
            .node_weights()
            .enumerate()
            .filter_map(|(idx, node_val)| match node_val {
                NodeValue::GateNode { .. } => Some((NodeIndex::new(idx), *node_val)),
                NodeValue::ValueNode(_) => None,
            })
    }

    pub fn add_node(&mut self, node: NodeUnitialised) -> NodeIndex {
        match node {
            NodeValue::GateNode { gate: g, .. } => self.graph.add_node(GraphNode::GateNode {
                gate: g,
                state_type: GateStatus::InvalidArity,
            }),
            NodeValue::ValueNode(v) => self.graph.add_node(NodeValue::ValueNode(v)),
        }
    }

    pub fn add_nodes(
        &mut self,
        nodes: impl Iterator<Item = NodeValue<NewNode>>,
    ) -> impl Iterator<Item = NodeIndex> {
        nodes
            .map(|x| self.add_node(x))
            .collect::<Box<[NodeIndex]>>()
            .into_iter()
    }

    pub fn update_node_status(
        &mut self,
        node_indx: NodeIndex,
    ) -> Result<NodeValue<GateStatus>, GraphError> {
        let incoming = self.get_neigh(node_indx, Direction::Incoming)?;
        let outgoing = self.get_neigh(node_indx, Direction::Outgoing)?;
        self.determine_node_status(node_indx, &incoming, &outgoing)
    }

    fn determine_node_status(
        &mut self,
        node_idx: NodeIndex,
        in_neigh: &[Value],
        out_neigh: &[Value],
    ) -> Result<NodeValue<GateStatus>, GraphError> {
        let Some(NodeValue::GateNode { gate, state_type }) = self.graph.node_weight_mut(node_idx)
        else {
            return Err(GraphError::NotExistentNode);
        };
        let gate = *gate;
        match gate.check(in_neigh, out_neigh) {
            Ok(b) => {
                *state_type = if b {
                    GateStatus::Valid
                } else {
                    GateStatus::InvalidValues
                }
            }
            Err(GateError::ArityError) => *state_type = GateStatus::InvalidArity,
            Err(GateError::NonDeterminsticGate) => {
                unreachable!("Check should not generate such error")
            }
        };
        Ok(NodeValue::<GateStatus>::GateNode {
            gate: gate,
            state_type: *state_type,
        })
    }

    #[inline]
    fn get_next_node(&self, indx: NodeIndex, dir: Direction) -> (u64, NodeIndex) {
        (
            self.graph
                .edges_directed(indx, dir)
                .map(|e| e.weight())
                .max()
                .copied()
                .unwrap_or(0)
                + 1,
            indx,
        )
    }

    #[inline]
    pub(crate) fn get_neigh(
        &self,
        indx: NodeIndex,
        dir: Direction,
    ) -> Result<Box<[Value]>, GraphError> {
        self.graph
            .neighbors_directed(indx, dir)
            .map(|m| {
                if let Some(NodeValue::ValueNode(ret)) = self.graph.node_weight(m).copied() {
                    Ok(ret)
                } else {
                    Err(GraphError::NonHeterogeneousEdge)
                }
            })
            .collect::<Result<_, _>>()
    }

    pub fn get_all_neigh(&self, indx: NodeIndex) -> Box<[NodeIndex]> {
        self.graph
            .neighbors_directed(indx, Direction::Incoming)
            .chain(self.graph.neighbors_directed(indx, Direction::Outgoing))
            .collect::<Box<[_]>>()
    }

    pub fn add_edge(
        &mut self,
        src_idx: NodeIndex,
        dest_idx: NodeIndex,
    ) -> Result<EdgeIndex, GraphError> {
        match (
            self.graph.node_weight(src_idx),
            self.graph.node_weight(dest_idx),
        ) {
            (Some(NodeValue::GateNode { .. }), Some(NodeValue::GateNode { .. }))
            | (Some(NodeValue::ValueNode(_)), Some(NodeValue::ValueNode(_))) => {
                return Err(GraphError::NonHeterogeneousEdge);
            }
            (None, _) | (_, None) => return Err(GraphError::NotExistentNode),
            _ => (),
        };

        let (value, gate_idx) = if matches!(
            self.graph
                .node_weight(src_idx)
                .ok_or(GraphError::NotExistentNode)?,
            NodeValue::ValueNode(_)
        ) {
            self.get_next_node(dest_idx, Direction::Incoming)
        } else {
            self.get_next_node(src_idx, Direction::Outgoing)
        };

        let ret = self.graph.add_edge(src_idx, dest_idx, value);
        self.update_node_status(gate_idx)?;

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::Gate;
    use crate::gates::Value;
    use crate::test_utils::enum_strategy;
    use anyhow::Result as ARes;
    use itertools::Itertools;
    use proptest::prelude::*;
    use proptest::strategy::BoxedStrategy;
    use std::fmt::Debug;

    #[allow(dead_code)]
    pub fn sample_from_slice<T: Debug + Clone + 'static>(values: &[T]) -> BoxedStrategy<T> {
        prop::sample::select(values.to_vec()).boxed()
    }

    mod node_tests {

        use crate::gates::NodeUnitialised;

        use super::*;
        proptest! {
            #[test]
            fn add_value_node(
                s in (prop::collection::linked_list(
                prop_oneof![
                    enum_strategy::<Value>()
                        .prop_map(NodeUnitialised::from_value),
                    enum_strategy::<Gate>()
                        .prop_map(NodeUnitialised::from_gate)
                ]
                , 5..1_000))
            ){
                let mut pc = PureCircuitGraph::default();
                let mut counter = 0;
                for n in s {
                    pc.add_node(n);
                    if let NodeValue::<NewNode>::GateNode {..} = n {
                        counter += 1;
                    }
                }
                let bad_nodes = pc.get_error_gates().count();

                prop_assert_eq!(counter, bad_nodes);
            }
        }
    }

    mod edge_tests {
        use std::collections::HashSet;

        use super::*;

        use crate::gates::GateStatus;

        fn generate_generic_graph()
        -> impl Strategy<Value = (HashSet<(usize, usize)>, Vec<NodeUnitialised>)> {
            prop::collection::vec(
                prop_oneof![
                    enum_strategy::<Value>().prop_map(NodeUnitialised::from_value),
                    enum_strategy::<Gate>().prop_map(NodeUnitialised::from_gate),
                ],
                1..=150,
            )
            .prop_flat_map(|arr| {
                (
                    prop::collection::hash_set((0..arr.len(), 0..arr.len()), 0..=150),
                    Just(arr),
                )
            })
        }

        fn generate_heterogeneous_graph() -> impl Strategy<
            Value = (
                Box<[(bool, usize, usize)]>,
                Vec<NodeUnitialised>,
                Vec<NodeUnitialised>,
            ),
        > {
            (
                prop::collection::vec(
                    enum_strategy::<Value>().prop_map(NodeUnitialised::from_value),
                    1..=5,
                ),
                prop::collection::vec(
                    enum_strategy::<Gate>().prop_map(NodeUnitialised::from_gate),
                    1..=5,
                ),
            )
                .prop_flat_map(|(arr_val, arr_gate)| {
                    (
                        (
                            prop::collection::btree_set(
                                (Just(true), 0..arr_val.len(), 0..arr_gate.len()),
                                0..=5,
                            ),
                            prop::collection::btree_set(
                                (Just(false), 0..arr_gate.len(), 0..arr_val.len()),
                                0..=5,
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

        #[test]
        fn test_simple_arity_incorrect_assignment() -> ARes<()> {
            let mut pc = PureCircuitGraph::default();
            let gt_indx = pc.add_node(NodeUnitialised::from_gate(Gate::Not));
            let val_indx = pc.add_node(NodeUnitialised::from_value(Value::Zero));
            pc.add_edge(val_indx, gt_indx)?;
            pc.add_edge(gt_indx, val_indx)?;
            let Some(NodeValue::GateNode {
                gate: _,
                state_type: status,
            }) = pc.graph.node_weight(gt_indx).copied()
            else {
                panic!("Ignore");
            };
            assert_eq!(status, GateStatus::InvalidValues);
            Ok(())
        }

        #[test]
        fn test_simple_arity_correct_assignment() -> ARes<()> {
            let mut pc = PureCircuitGraph::default();
            let gt_indx = pc.add_node(NodeUnitialised::from_gate(Gate::Not));
            let val_indx = pc.add_node(NodeUnitialised::from_value(Value::Bot));
            pc.add_edge(val_indx, gt_indx)?;
            pc.add_edge(gt_indx, val_indx)?;

            let Some(NodeValue::GateNode {
                state_type: status, ..
            }) = pc.graph.node_weight(gt_indx).copied()
            else {
                panic!("Ignore");
            };
            assert_eq!(status, GateStatus::Valid);
            Ok(())
        }

        #[test]
        fn test_duplicates() {
            // Prepare
            let mut pc = PureCircuitGraph::default();
            let gate_idx = pc.add_node(NodeUnitialised::from_gate(Gate::And));
            let val_idx = pc.add_node(NodeUnitialised::from_value(Value::Bot));
            let edge1 = pc.add_edge(gate_idx, val_idx);
            assert!(matches!(edge1, Ok(_)));
            let w = pc.graph.edge_weight(edge1.unwrap()).copied().unwrap();
            let edge2 = pc.add_edge(gate_idx, val_idx);
            assert!(matches!(edge2, Ok(_)));
            let w2 = pc.graph.edge_weight(edge2.unwrap()).copied().unwrap();
            assert!(w < w2)
        }

        proptest! {
            #[test]
            fn check_invalid_edges(s in generate_generic_graph()) {
                let mut graph = PureCircuitGraph::default();
                let (edges, array) = s;
                let node_indxes = graph.add_nodes(array.iter().cloned()).collect_vec();
                for (src, dst) in edges {
                    let flag = match (array[src], array[dst]) {
                        (NodeUnitialised::GateNode { .. }, NodeUnitialised::ValueNode(_))
                        | (NodeUnitialised::ValueNode(_), NodeUnitialised::GateNode { .. }) => true,
                        _ => false,
                    };
                    let result = graph.add_edge(node_indxes[src], node_indxes[dst]);
                    prop_assert!(
                        flag == matches!(result, Ok(_))
                    , "Output {:?} {:?}. Edge ({:?}, {:?})", result, flag, node_indxes[src], node_indxes[dst]);
                    prop_assert!(
                        !flag == matches!(result, Err(GraphError::NonHeterogeneousEdge))
                    , "Output {:?} {:?}. Edge ({:?}, {:?})", result, flag, node_indxes[src], node_indxes[dst]);
                    graph.graph.edges_connecting(node_indxes[src], node_indxes[dst]).count();
                }
            }

            #[test]
            fn check_bad_nodes(s in generate_heterogeneous_graph()) {
                let mut pc = PureCircuitGraph::default();
                let (edges, vals, gates) = s;
                let vals_idx = pc.add_nodes(vals.into_iter()).collect_vec();
                let gates_idx = pc.add_nodes(gates.into_iter()).collect_vec();
                let mut checks = true;
                for (dir, src, dest) in edges {
                    if dir {
                        checks &= pc.add_edge(vals_idx[src], gates_idx[dest]).is_ok();
                    } else {
                        checks &= pc.add_edge(gates_idx[src], vals_idx[dest]).is_ok();
                    }
                }
                prop_assert!(checks);
                for node_ix in pc.graph.node_indices() {
                    let node_weight = pc.graph.node_weight(node_ix);
                    prop_assert!(node_weight.is_some());
                    let node_weight = node_weight.unwrap();
                    match node_weight {
                        GraphNode::ValueNode(_) => continue,
                        GraphNode::GateNode { gate, state_type: status } => {
                            let in_neigh = pc.graph.neighbors_directed(node_ix,Direction::Incoming).map(|ind| pc.graph.node_weight(ind).unwrap()).collect_vec();
                            let out_neigh = pc.graph.neighbors_directed(node_ix,Direction::Outgoing).map(|ind| pc.graph.node_weight(ind).unwrap()).collect_vec();
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
                                prop_assert_ne!(*status, GateStatus::InvalidArity,
                                    "Status received {:?}, for gate {:?} with arity {:?}",
                                    status,
                                    gate,
                                    gate.arity()
                                );
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
