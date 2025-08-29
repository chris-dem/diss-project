use petgraph::{prelude::*, stable_graph::StableDiGraph, visit::IntoEdgeReferences};
use std::fmt::Debug;
use strum_macros::Display;

use crate::gates::{
    GateError, GateStatus, GraphNode, GraphStruct, NewNode, NodeUnitialised, NodeValue, Value,
};

pub type BoxArray<T> = Box<[T]>;

#[derive(Debug, Clone)]
pub struct PureCircuitGraph<T = (), G = ()> {
    pub graph: StableDiGraph<GraphStruct<T>, (u64, G)>,
}

impl<T: Default + Debug, G: Debug> Default for PureCircuitGraph<T, G> {
    fn default() -> Self {
        Self {
            graph: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum GraphError {
    NotExistentNode,
    NotExistentEdge,
    NonHeterogeneousEdge,
    InvalidUpdate,
}

impl std::error::Error for GraphError {}

impl<T, G> PureCircuitGraph<T, G> {
    pub fn count_values(&self) -> usize {
        self.graph
            .node_weights()
            .filter(|i| !i.node.is_gate())
            .count()
    }
}

impl<T: Debug + Default, G: Debug> PureCircuitGraph<T, G> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Copy, G: Copy> PureCircuitGraph<T, G> {
    /// Get all sources, targets and edge weights of the graph
    pub fn get_edges(&self) -> impl Iterator<Item = (NodeIndex, NodeIndex, (u64, G))> {
        self.graph
            .edge_references()
            .map(|r| (r.source(), r.target(), *r.weight()))
    }

    /// Numbers of value nodes
    pub fn get_value_count(&self) -> usize {
        self.graph
            .node_weights()
            .filter(|e| matches!(e.into_node(), NodeValue::ValueNode(_)))
            .count()
    }

    /// Updates the value of a node in the graph.
    ///
    /// # Parameters
    /// * `index` - The index of the node to update
    /// * `new_value` - The new value to assign to the node
    ///
    /// # Returns
    /// A boxed array of node indices that were affected by the update.
    ///
    /// # Errors
    /// Returns `GraphError` if the node index is invalid or the update fails.
    pub fn update_node(
        &mut self,
        index: NodeIndex,
        new_value: NodeUnitialised,
    ) -> Result<Box<[NodeIndex]>, GraphError> {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(GraphError::NotExistentNode)?;

        let copied_node = node.node;
        match (&mut node.node, new_value) {
            (NodeValue::GateNode { gate, .. }, NodeValue::GateNode { gate: g, .. }) => *gate = g,
            (NodeValue::ValueNode(val), NodeValue::ValueNode(e)) => *val = e,
            _ => return Err(GraphError::InvalidUpdate),
        };

        match copied_node {
            NodeValue::GateNode { .. } => {
                self.update_node_status(index)?;
                Ok(Box::new([index]))
            }
            NodeValue::ValueNode(_) => {
                let ret = self.get_all_neigh(index);
                for n in ret.iter().copied() {
                    self.update_node_status(n)?;
                }
                Ok(ret)
            }
        }
    }

    /// Updates the value of a node in the graph.
    ///
    /// # Returns
    /// Iterator of indexes of all invalid gates
    pub fn get_error_gates(&self) -> impl Iterator<Item = NodeIndex> {
        self.graph
            .node_weights()
            .map(GraphStruct::into_node)
            .enumerate()
            .filter_map(|(idx, node_val)| match node_val {
                GraphNode::GateNode { .. } => Some(NodeIndex::new(idx)),
                NodeValue::ValueNode(_) => None,
            })
    }

    /// Add new nodein graph
    ///
    /// # Parameters
    /// * `node` - Uninitialised node
    /// * `additional_info` - Additional information of node
    ///
    /// # Returns
    /// Index of the new node
    pub fn add_node(&mut self, node: NodeUnitialised, additional_info: T) -> NodeIndex {
        match node {
            NodeValue::GateNode { gate: g, .. } => self.graph.add_node(GraphStruct {
                node: GraphNode::GateNode {
                    gate: g,
                    state_type: GateStatus::InvalidArity,
                },
                additional_info,
            }),
            NodeValue::ValueNode(v) => self.graph.add_node(GraphStruct::<T>::new(
                NodeValue::ValueNode(v),
                additional_info,
            )),
        }
    }

    pub(crate) fn add_nodes(
        &mut self,
        nodes: impl Iterator<Item = (NodeValue<NewNode>, T)>,
    ) -> impl Iterator<Item = NodeIndex> {
        nodes
            .map(|x| self.add_node(x.0, x.1))
            .collect::<Box<[NodeIndex]>>()
            .into_iter()
    }

    /// Updates node status of gate
    ///
    /// # Parameters
    /// * `node_indx` - Index of gate node
    ///
    /// # Returns
    /// State of the new gate node
    ///
    /// # Errors
    /// NotExistentNode: If current node is missing or value is not gate
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
        let Some(GraphStruct {
            node: NodeValue::GateNode { gate, state_type },
            ..
        }) = self.graph.node_weight_mut(node_idx)
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
            gate,
            state_type: *state_type,
        })
    }

    #[inline]
    fn get_next_node(&self, indx: NodeIndex, dir: Direction) -> (u64, NodeIndex) {
        (
            self.graph
                .edges_directed(indx, dir)
                .map(|e| e.weight().0)
                .max()
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
            .edges_directed(indx, dir)
            .map(|m| {
                let n = match dir {
                    Direction::Incoming => m.source(),
                    Direction::Outgoing => m.target(),
                };
                if let Some(NodeValue::ValueNode(ret)) =
                    self.graph.node_weight(n).map(GraphStruct::into_node)
                {
                    Ok((m.weight().0, ret))
                } else {
                    Err(GraphError::NonHeterogeneousEdge)
                }
            })
            .collect::<Result<Box<[(u64, Value)]>, GraphError>>()
            .map(|mut v| {
                v.sort_by_key(|t| t.0);
                v.into_iter().map(|t| t.1).collect::<Box<[Value]>>()
            })
    }

    /// Add edge between two nodes
    /// Ensures that the gate status will be updated
    ///
    /// # Parameters
    /// * `src_indx` - Index of source node
    /// * `dest_indx` - Index of dest node
    ///
    /// # Returns
    /// Gate index, EdgeIndex, value of node
    ///
    /// # Errors
    /// * NotHeterogeneousEdge: Cannot add edge between value value of gate gate
    /// * NotExistentNode: Cannot add edge between non existent nodes
    pub fn add_edge(
        &mut self,
        src_idx: NodeIndex,
        dest_idx: NodeIndex,
        additional_info: G,
    ) -> Result<(NodeIndex, EdgeIndex, u64), GraphError> {
        match (
            self.graph.node_weight(src_idx).map(GraphStruct::into_node),
            self.graph.node_weight(dest_idx).map(GraphStruct::into_node),
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
                .map(GraphStruct::into_node)
                .ok_or(GraphError::NotExistentNode)?,
            NodeValue::ValueNode(_)
        ) {
            self.get_next_node(dest_idx, Direction::Incoming)
        } else {
            self.get_next_node(src_idx, Direction::Outgoing)
        };

        let ret = self
            .graph
            .add_edge(src_idx, dest_idx, (value, additional_info));
        self.update_node_status(gate_idx)?;

        Ok((gate_idx, ret, value))
    }

    /// Remove edge between two nodes. Ensures that the gate status will be updated.
    ///
    /// # Parameters
    /// * `src_indx` - Index of source node
    /// * `dest_indx` - Index of dest node
    ///
    /// # Returns
    /// Gate index, EdgeIndex, value of node
    ///
    /// # Errors
    /// * NotHeterogeneousEdge: Cannot add edge between value value of gate gate
    /// * NotExistentNode: Cannot add edge between non existent nodes
    pub fn remove_edge(
        &mut self,
        src_idx: NodeIndex,
        dest_idx: NodeIndex,
    ) -> Result<(NodeIndex, BoxArray<(u64, G)>), GraphError> {
        match (
            self.graph.node_weight(src_idx).map(GraphStruct::into_node),
            self.graph.node_weight(dest_idx).map(GraphStruct::into_node),
        ) {
            (Some(NodeValue::GateNode { .. }), Some(NodeValue::GateNode { .. }))
            | (Some(NodeValue::ValueNode(_)), Some(NodeValue::ValueNode(_))) => {
                return Err(GraphError::NonHeterogeneousEdge);
            }
            (None, _) | (_, None) => return Err(GraphError::NotExistentNode),
            _ => (),
        };

        let edges = self
            .graph
            .edges_connecting(src_idx, dest_idx)
            .map(|e| e.id())
            .collect::<Box<[_]>>();
        if edges.len() == 0 {
            return Err(GraphError::NotExistentEdge);
        }

        let edges = edges
            .into_iter()
            .map(|e| self.graph.remove_edge(e))
            .collect::<Option<Box<[(u64, G)]>>>()
            .ok_or(GraphError::NotExistentEdge)?;

        let gate_idx = if matches!(
            self.graph
                .node_weight(src_idx)
                .map(GraphStruct::into_node)
                .ok_or(GraphError::NotExistentNode)?,
            NodeValue::ValueNode(_)
        ) {
            dest_idx
        } else {
            src_idx
        };

        self.update_node_status(gate_idx)?;

        Ok((gate_idx, edges))
    }

    /// Remove node. If it is a value node, update the status of all its neighbours
    ///
    /// # Parameters
    /// * `nod_indx` - Index of source node
    ///
    /// # Returns
    /// Set of gates that have been affected
    ///
    /// # Errors
    /// * NotExistentNode: If node index does not exist
    pub fn remove_node(&mut self, node_idx: NodeIndex) -> Result<Box<[NodeIndex]>, GraphError> {
        let weight = self
            .graph
            .node_weight(node_idx)
            .copied()
            .ok_or(GraphError::NotExistentNode)?;

        let neigh = match weight.into_node() {
            NodeValue::ValueNode(_) => Some(self.get_all_neigh(node_idx)),
            NodeValue::GateNode { .. } => None,
        };

        self.graph
            .remove_node(node_idx)
            .ok_or(GraphError::NotExistentNode)?;

        Ok(neigh.unwrap_or_default())
    }
}

impl<T, G> PureCircuitGraph<T, G> {
    /// Get all neighbours of node
    ///
    /// # Parameters
    /// * `indx` - Index of source node
    ///
    /// # Returns
    /// Set of gates that have been affected
    pub fn get_all_neigh(&self, indx: NodeIndex) -> Box<[NodeIndex]> {
        self.graph
            .neighbors_directed(indx, Direction::Incoming)
            .chain(self.graph.neighbors_directed(indx, Direction::Outgoing))
            .collect::<Box<[_]>>()
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
                let mut pc = PureCircuitGraph::<(), ()>::default();
                let mut counter = 0;
                for n in s {
                    pc.add_node(n, ());
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
            let mut pc = PureCircuitGraph::<(), ()>::default();
            let gt_indx = pc.add_node(NodeUnitialised::from_gate(Gate::Not), ());
            let val_indx = pc.add_node(NodeUnitialised::from_value(Value::Zero), ());
            pc.add_edge(val_indx, gt_indx, ())?;
            pc.add_edge(gt_indx, val_indx, ())?;
            let Some(NodeValue::GateNode {
                gate: _,
                state_type: status,
            }) = pc.graph.node_weight(gt_indx).map(GraphStruct::into_node)
            else {
                panic!("Ignore");
            };
            assert_eq!(status, GateStatus::InvalidValues);
            Ok(())
        }

        #[test]
        fn test_simple_arity_correct_assignment() -> ARes<()> {
            let mut pc = PureCircuitGraph::<(), ()>::default();
            let gt_indx = pc.add_node(NodeUnitialised::from_gate(Gate::Not), ());
            let val_indx = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            pc.add_edge(val_indx, gt_indx, ())?;
            pc.add_edge(gt_indx, val_indx, ())?;

            let Some(NodeValue::GateNode {
                state_type: status, ..
            }) = pc.graph.node_weight(gt_indx).map(GraphStruct::into_node)
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
            let gate_idx = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_idx = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let (i1, _, w1) = pc.add_edge(gate_idx, val_idx, ()).expect("No errors");
            assert_eq!(gate_idx, i1);
            let (i2, _, w2) = pc.add_edge(gate_idx, val_idx, ()).expect("No errors");
            assert_eq!(gate_idx, i2);
            assert!(w1 < w2)
        }

        proptest! {
            #[test]
            fn check_invalid_edges(s in generate_generic_graph()) {
                let mut graph = PureCircuitGraph::<(), ()>::default();
                let (edges, array) = s;
                let node_indxes = graph.add_nodes(array.iter().cloned().map(|x| (x, ()))).collect_vec();
                for (src, dst) in edges {
                    let flag = match (array[src], array[dst]) {
                        (NodeUnitialised::GateNode { .. }, NodeUnitialised::ValueNode(_))
                        | (NodeUnitialised::ValueNode(_), NodeUnitialised::GateNode { .. }) => true,
                        _ => false,
                    };
                    let result = graph.add_edge(node_indxes[src], node_indxes[dst], ());
                    prop_assert!(
                        flag == result.is_ok()
                    , "Output {:?} {:?}. Edge ({:?}, {:?})", result, flag, node_indxes[src], node_indxes[dst]);
                    prop_assert!(
                        flag != matches!(result, Err(GraphError::NonHeterogeneousEdge))
                    , "Output {:?} {:?}. Edge ({:?}, {:?})", result, flag, node_indxes[src], node_indxes[dst]);
                    graph.graph.edges_connecting(node_indxes[src], node_indxes[dst]).count();
                }
            }

            #[test]
            fn check_bad_nodes(s in generate_heterogeneous_graph()) {
                let mut pc = PureCircuitGraph::<(), ()>::default();
                let (edges, vals, gates) = s;
                let vals_idx = pc.add_nodes(vals.into_iter().map(|x| (x, ()))).collect_vec();
                let gates_idx = pc.add_nodes(gates.into_iter().map(|x| (x, ()))).collect_vec();
                let mut checks = true;
                for (dir, src, dest) in edges {
                    if dir {
                        checks &= pc.add_edge(vals_idx[src], gates_idx[dest], ()).is_ok();
                    } else {
                        checks &= pc.add_edge(gates_idx[src], vals_idx[dest], ()).is_ok();
                    }
                }
                prop_assert!(checks);
                for node_ix in pc.graph.node_indices() {
                    let node_weight = pc.graph.node_weight(node_ix);
                    prop_assert!(node_weight.is_some());
                    let node_weight = node_weight.unwrap();
                    match node_weight.into_node() {
                        GraphNode::ValueNode(_) => continue,
                        GraphNode::GateNode { gate, state_type: status } => {
                            let mut in_neigh = pc.graph.edges_directed(node_ix,Direction::Incoming)
                                .map(|e| (e.weight().0, pc.graph.node_weight(e.source()).unwrap().into_node())).collect_vec();
                            in_neigh.sort_by_key(|e| e.0);
                            let in_neigh = in_neigh.into_iter().map(|x| x.1).collect_vec();
                            let mut out_neigh = pc.graph.edges_directed(node_ix,Direction::Outgoing)
                                .map(|e| (e.weight().0, pc.graph.node_weight(e.target()).unwrap().into_node())).collect_vec();
                            out_neigh.sort_by_key(|e| e.0);
                            let out_neigh = out_neigh.into_iter().map(|x| x.1).collect_vec();
                            prop_assert!(in_neigh.iter().all(|f| matches!(f,NodeValue::ValueNode(_))));
                            prop_assert!(out_neigh.iter().all(|f| matches!(f,NodeValue::ValueNode(_))));
                            let in_neigh = in_neigh.into_iter().map(|el| {
                                let NodeValue::ValueNode(v) = el else { panic!("Should not happen")};
                                v
                            }).collect_vec();
                            let out_neigh = out_neigh.into_iter().map(|el| {
                                let NodeValue::ValueNode(v) = el else { panic!("Should not happen")};
                                v
                            }).collect_vec();
                            if (in_neigh.len(),out_neigh.len()) == gate.arity() {
                                prop_assert_ne!(status, GateStatus::InvalidArity,
                                    "Status received {:?}, for gate {:?} with arity {:?}",
                                    status,
                                    gate,
                                    gate.arity()
                                );
                                match gate.check(&in_neigh, &out_neigh) {
                                    Ok(b) => {
                                        prop_assert!(b == (status == GateStatus::Valid), "Status {status:?} {gate:?} {in_neigh:?} {out_neigh:?}")
                                    },
                                    Err(_) => prop_assert!(false, "Should not reach errored gate"),
                                }
                            } else {
                                prop_assert_eq!(status, GateStatus::InvalidArity);
                            }
                        },
                    }
                }
            }


        }
    }
}
