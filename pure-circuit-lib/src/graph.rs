use crate::gates::{GateCheck, NodeValue};
use itertools::Itertools;
use petgraph::{data::Build, graphmap::EdgesDirected, prelude::*};

#[derive(Debug, Clone)]
pub struct PureCircuitGraph {
    pub graph: DiGraph<NodeValue, ()>,
    pub bad_nodes: Vec<NodeIndex>,
    pub bad_values: Vec<NodeIndex>,
}

impl Default for PureCircuitGraph {
    fn default() -> Self {
        Self {
            graph: Default::default(),
            bad_nodes: Vec::new(),
            bad_values: Vec::new(),
        }
    }
}

enum PureCircuitErrors {
    MismatchedEdges,
    MissingNodes,
}

impl PureCircuitGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: NodeValue) -> NodeIndex {
        let ret = self.graph.add_node(node);
        if let NodeValue::GateNode(_) = node {
            self.bad_nodes.push(ret);
        }
        ret
    }

    pub fn add_edge(
        &mut self,
        src_idx: NodeIndex,
        dest_idx: NodeIndex,
    ) -> Result<EdgeIndex, PureCircuitErrors> {
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
    use crate::gates::Value;
    use itertools::Itertools;
    use proptest::prelude::*;
    use proptest::strategy::BoxedStrategy;
    use std::fmt::Debug;
    use strum::IntoEnumIterator;

    pub fn sample_from_slice<T: Debug + Clone + 'static>(values: &[T]) -> BoxedStrategy<T> {
        prop::sample::select(values.to_vec()).boxed()
    }

    mod node_tests {

        use crate::gates::Gate;

        use super::*;

        proptest! {
            #[test]
            fn add_value_node(
                s in (prop::collection::linked_list(
                sample_from_slice(&Value::iter()
                    .map(NodeValue::ValueNode)
                    .chain(Gate::iter().map(NodeValue::GateNode)).collect_vec()
            ), 5..1_000))
            ){
                let mut pc = PureCircuitGraph::default();
                let mut counter = 0;
                for n in s {
                    pc.add_node(n);
                    if let NodeValue::GateNode(_) = n {
                        counter += 1;
                    }
                }
                prop_assert_eq!(counter, pc.bad_nodes.len());
            }
        }
    }
}
