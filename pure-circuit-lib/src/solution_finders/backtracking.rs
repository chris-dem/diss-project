use std::fmt::Debug;

use crate::{
    gates::{GraphNode, NodeUnitialised, NodeValue, Value, VoltageOrdering},
    graph::PureCircuitGraph,
    solution_finders::base_finder::MAX_DEGREE,
};
use anyhow::{Result as ARes, anyhow};
use itertools::Itertools;
use petgraph::{Direction, prelude::NodeIndex, visit::EdgeRef};
use priority_queue::PriorityQueue;
use strum::IntoEnumIterator;

pub struct BacktrackAlgorithm;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct BitString(pub(crate) bool, pub(crate) bool, pub(crate) bool);

impl Debug for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self.into_values();
        write!(f, "{x:?}")
    }
}

impl TryFrom<u8> for BitString {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 8 {
            Ok(BitString(
                (value & 0b1) == 1,
                (value & 0b10) == 2,
                (value & 0b100) == 4,
            ))
        } else {
            ARes::Err(anyhow!("Cannot convert to bistring"))
        }
    }
}

impl From<Value> for BitString {
    fn from(value: Value) -> Self {
        let val = match value {
            Value::Zero => 1,
            Value::Bot => 2,
            Value::One => 4,
        };
        Self::try_from(val).unwrap()
    }
}

impl BitString {
    pub(crate) fn len(self) -> usize {
        self.0 as usize + self.1 as usize + self.2 as usize
    }

    pub(crate) fn to_value_iter(self) -> impl Iterator<Item = Value> {
        let mut ret = Vec::with_capacity(3);
        if self.0 {
            ret.push(Value::Zero);
        }
        if self.1 {
            ret.push(Value::Bot);
        }
        if self.2 {
            ret.push(Value::One);
        }
        ret.into_iter()
    }
    pub(crate) fn all() -> Self {
        Self(true, true, true)
    }
    fn into_values(self) -> Vec<Value> {
        [
            if self.0 { Some(Value::Zero) } else { None },
            if self.1 { Some(Value::Bot) } else { None },
            if self.2 { Some(Value::One) } else { None },
        ]
        .into_iter()
        .flatten()
        .collect_vec()
    }

    pub fn op_union(self, other: Self) -> Self {
        Self(self.0 || other.0, self.1 || other.1, self.2 || other.2)
    }

    pub fn op_inter(self, other: Self) -> Self {
        Self(self.0 && other.0, self.1 && other.1, self.2 && other.2)
    }

    pub fn add(self, value: Value) -> Self {
        self.op_union(BitString::from(value))
    }

    pub fn reverse(self) -> Self {
        Self(!self.0, !self.1, !self.2)
    }

    pub fn flip(self) -> Self {
        Self(self.2, self.1, self.0)
    }

    pub fn remove(self, value: Value) -> Self {
        self.op_inter(BitString::from(value).reverse())
    }

    pub fn smaller_than(value: Value) -> Self {
        Value::iter()
            .filter(|x| VoltageOrdering(*x) <= VoltageOrdering(value))
            .fold(BitString::default(), |acc, x| acc.add(x))
    }

    pub fn greater_than(value: Value) -> Self {
        Self::smaller_than(value).reverse().add(value)
    }
}

impl<T: Copy, G: Copy> PureCircuitGraph<T, G> {
    pub fn from_backtrack_sol(&mut self, v: &[Option<Value>]) -> ARes<()> {
        for n in self.graph.node_indices().collect_vec() {
            if self
                .graph
                .node_weight(n)
                .filter(|f| matches!(f.node, NodeValue::ValueNode(_)))
                .is_none()
            {
                continue;
            }
            let Some(Some(new_val)) = v.get(n.index()) else {
                return Err(anyhow!("Index missmatch backtrack"));
            };
            self.update_node(n, NodeUnitialised::from_value(*new_val))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BacktrackKey {
    value_len: usize,
    is_start: bool,
    neighbours: usize,
}

impl PartialOrd for BacktrackKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (
                other.value_len as isize,
                other.is_start as usize,
                other.neighbours,
            )
                .cmp(&(
                    self.value_len as isize,
                    self.is_start as usize,
                    self.neighbours,
                )),
        )
    }
}

impl Ord for BacktrackKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

type BacktrackPQ = PriorityQueue<NodeIndex, BacktrackKey>;

impl<T, G> PureCircuitGraph<T, G> {
    fn extract_node_graph(&self) -> Vec<Option<(NodeIndex, bool, usize)>> {
        let v = self
            .graph
            .node_indices()
            .filter_map(|n| {
                let node = self.graph[n].into_node();
                if matches!(node, NodeValue::GateNode { .. }) {
                    return None;
                }
                let in_degree = self
                    .graph
                    .neighbors_directed(n, Direction::Incoming)
                    .count();
                Some((
                    n,
                    in_degree == 0,
                    in_degree
                        + self
                            .graph
                            .neighbors_directed(n, Direction::Outgoing)
                            .count(),
                ))
            })
            .collect_vec();
        let max_ind = v.iter().map(|i| i.0.index()).max().unwrap();
        let mut ret = vec![None; max_ind + 1];
        for el in v {
            ret[el.0.index()] = Some(el);
        }
        ret
    }
}

impl BacktrackAlgorithm {
    pub fn calculate<T, G>(
        &self,
        pc_instance: &PureCircuitGraph<T, G>,
    ) -> ARes<Vec<Vec<Option<Value>>>> {
        let node_array = pc_instance.extract_node_graph(); // We want to get value nodes
        let mut value_map = vec![None; node_array.len()];
        for (el_node, el_val) in node_array.iter().zip(value_map.iter_mut()) {
            if el_node.is_some() {
                *el_val = Some(BitString::all());
            }
        }
        let mut queue: BacktrackPQ = PriorityQueue::new();
        for i in node_array.iter().filter_map(|x| *x) {
            queue.push(
                i.0,
                BacktrackKey {
                    value_len: 3,
                    is_start: i.1,
                    neighbours: i.2,
                },
            );
        }
        self.backtrack_root(pc_instance, value_map, queue)
    }

    fn backtrack_root<T, G>(
        &self,
        pc_instance: &PureCircuitGraph<T, G>,
        value_map: Vec<Option<BitString>>,
        mut queue: BacktrackPQ,
    ) -> ARes<Vec<Vec<Option<Value>>>> {
        let Some((val_ind, _)) = queue.pop() else {
            return Err(anyhow!("Empty graph"));
        };

        let mut count = vec![];
        for v in value_map[val_ind.index()]
            .ok_or(anyhow!("Node mapping incorrect"))?
            .to_value_iter()
        {
            let mut sol_arr = vec![None; value_map.len()];
            let mut val_map = value_map.clone();
            let mut q = queue.clone();
            let res =
                self.unit_propagate(val_ind, v, &mut sol_arr, &mut val_map, &mut q, pc_instance)?;
            if !res {
                continue;
            }

            let mut res = self.backtrack_inner(pc_instance, val_map, q, sol_arr)?;
            count.append(&mut res);
        }

        Ok(count)
    }

    fn unit_propagate<T, G>(
        &self,
        index_pc: NodeIndex,
        new_value: Value,
        sol_map: &mut [Option<Value>],
        value_map: &mut [Option<BitString>],
        queue: &mut BacktrackPQ,
        pc_instance: &PureCircuitGraph<T, G>,
    ) -> ARes<bool> {
        self.assign_node(index_pc, new_value, sol_map, value_map, queue)?;
        self.propagate_value_node(index_pc, sol_map, value_map, queue, pc_instance)?;
        while queue.peek().filter(|(_, p)| p.value_len == 1).is_some() {
            let (nod_ind, _k) = queue.pop().unwrap();
            let b = value_map[nod_ind.index()].ok_or(anyhow!("Not exist"))?;
            if b.len() != 1 {
                let el = b.len();
                let el2 = _k.value_len;

                return Err(anyhow!(
                    "Incorrect measurement: Len {el}, Key {el2}, Map {value_map:?}, Ind {nod_ind:?}"
                ));
            }
            let new_val = b.to_value_iter().next().unwrap();
            self.assign_node(nod_ind, new_val, sol_map, value_map, queue)?;
            self.propagate_value_node(nod_ind, sol_map, value_map, queue, pc_instance)?;
        }

        if queue.is_empty() {
            return Ok(true);
        }
        if queue.peek().filter(|(_, p)| p.value_len == 0).is_some() {
            return Ok(false);
        }

        Ok(true)
    }

    fn propagate_value_node<T, G>(
        &self,
        index_pc: NodeIndex,
        sol_map: &mut [Option<Value>],
        value_map: &mut [Option<BitString>],
        queue: &mut BacktrackPQ,
        pc_instance: &PureCircuitGraph<T, G>,
    ) -> ARes<()> {
        for gate_indx in pc_instance.get_all_neigh(index_pc) {
            let mut ins_vals = [None; MAX_DEGREE];
            let mut ins_idx = [None; MAX_DEGREE];
            let mut outs_vals = [None; MAX_DEGREE];
            let mut outs_idx = [None; MAX_DEGREE];

            for outer_e in pc_instance
                .graph
                .edges_directed(gate_indx, Direction::Outgoing)
            {
                let ind = (outer_e.weight().0 as usize)
                    .checked_sub(1)
                    .ok_or(anyhow!("Index issue"))?;
                let val = outer_e.target();
                outs_vals[ind] = sol_map[val.index()];
                outs_idx[ind] = Some(val.index());
            }
            for outer_e in pc_instance
                .graph
                .edges_directed(gate_indx, Direction::Incoming)
            {
                let ind = (outer_e.weight().0 as usize)
                    .checked_sub(1)
                    .ok_or(anyhow!("Index issue"))?;
                let val = outer_e.source();
                ins_vals[ind] = sol_map[val.index()];
                ins_idx[ind] = Some(val.index());
            }
            let GraphNode::GateNode { gate, .. } = pc_instance.graph[gate_indx].into_node() else {
                panic!("error mappings");
            };
            let (checked_ins, checked_outs) = gate.set_value(ins_vals, outs_vals)?;
            for f in checked_ins
                .into_iter()
                .zip(ins_idx.into_iter())
                .chain(checked_outs.into_iter().zip(outs_idx.into_iter()))
            {
                if let (Some(val), Some(indx)) = f {
                    if sol_map[indx].is_none() {
                        self.prop_node(indx, val, queue, value_map)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn assign_node(
        &self,
        node_indx: NodeIndex,
        new_value: Value,
        sol_map: &mut [Option<Value>],
        value_map: &mut [Option<BitString>],
        queue: &mut BacktrackPQ,
    ) -> ARes<()> {
        if value_map[node_indx.index()].is_some() {
            sol_map[node_indx.index()] = Some(new_value);
            value_map[node_indx.index()] = Some(new_value.into());
            queue.remove(&node_indx);
            Ok(())
        } else {
            Err(anyhow!("Incorrect Assingment"))
        }
    }

    fn prop_node(
        &self,
        indx: usize,
        new_set: BitString,
        queue: &mut BacktrackPQ,
        value_map: &mut [Option<BitString>],
    ) -> ARes<()> {
        // dbg!(&value_map, &queue, &indx, &new_set);
        if value_map[indx].is_none() {
            return Err(anyhow!("Value mapping"));
        }
        let new_set = value_map[indx].unwrap().op_inter(new_set);
        value_map[indx] = Some(new_set);
        queue.change_priority_by(&NodeIndex::from(indx as u32), |f| {
            f.value_len = new_set.len();
        });
        Ok(())
    }

    fn backtrack_inner<T, G>(
        &self,
        pc_instance: &PureCircuitGraph<T, G>,
        value_map: Vec<Option<BitString>>,
        mut queue: BacktrackPQ,
        sol_map: Vec<Option<Value>>,
    ) -> ARes<Vec<Vec<Option<Value>>>> {
        let Some((val_ind, _)) = queue.pop() else {
            return Ok(vec![sol_map]);
        };
        let mut count = vec![];
        for v in value_map[val_ind.index()]
            .ok_or(anyhow!("Node mapping incorrect"))?
            .to_value_iter()
        {
            let mut sol_arr = sol_map.clone();
            let mut val_map = value_map.clone();
            let mut q = queue.clone();
            let res =
                self.unit_propagate(val_ind, v, &mut sol_arr, &mut val_map, &mut q, pc_instance)?;
            if !res {
                continue;
            }
            let mut res = self.backtrack_inner(pc_instance, val_map, q, sol_arr)?;
            count.append(&mut res);
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use crate::gates::{Gate, NodeUnitialised};

    use super::*;
    mod test_small_circuits {
        use super::*;

        #[test]
        fn test_copy() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g = pc.add_node(NodeUnitialised::from_gate(Gate::Copy), ());
            pc.add_edge(v1, g, ()).unwrap();
            pc.add_edge(g, v2, ()).unwrap();

            let back = BacktrackAlgorithm.calculate(&pc).expect("Should be valid");
            assert_eq!(3, back.len())
        }

        #[test]
        fn test_not() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g = pc.add_node(NodeUnitialised::from_gate(Gate::Not), ());
            pc.add_edge(v1, g, ()).unwrap();
            pc.add_edge(g, v2, ()).unwrap();

            let back = BacktrackAlgorithm.calculate(&pc).expect("Should be valid");
            assert_eq!(3, back.len())
        }

        #[test]
        fn test_purify() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v3 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g = pc.add_node(NodeUnitialised::from_gate(Gate::Purify), ());
            pc.add_edge(v1, g, ()).unwrap();
            pc.add_edge(g, v2, ()).unwrap();
            pc.add_edge(g, v3, ()).unwrap();

            let back = BacktrackAlgorithm.calculate(&pc).expect("Should be valid");
            assert_eq!(5, back.len())
        }

        #[test]
        fn test_and() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v3 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            pc.add_edge(v1, g, ()).unwrap();
            pc.add_edge(v2, g, ()).unwrap();
            pc.add_edge(g, v3, ()).unwrap();

            let back = BacktrackAlgorithm.calculate(&pc).expect("Should be valid");
            assert_eq!(9, back.len())
        }

        #[test]
        fn test_or() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v3 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g = pc.add_node(NodeUnitialised::from_gate(Gate::Or), ());
            pc.add_edge(v1, g, ()).unwrap();
            pc.add_edge(v2, g, ()).unwrap();
            pc.add_edge(g, v3, ()).unwrap();

            let back = BacktrackAlgorithm.calculate(&pc).expect("Should be valid");
            assert_eq!(9, back.len())
        }

        #[test]
        fn test_nand() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v3 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g = pc.add_node(NodeUnitialised::from_gate(Gate::Nand), ());
            pc.add_edge(v1, g, ()).unwrap();
            pc.add_edge(v2, g, ()).unwrap();
            pc.add_edge(g, v3, ()).unwrap();

            let back = BacktrackAlgorithm.calculate(&pc).expect("Should be valid");
            assert_eq!(9, back.len())
        }

        #[test]
        fn test_nor() {
            let mut pc: PureCircuitGraph = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v3 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g = pc.add_node(NodeUnitialised::from_gate(Gate::Nor), ());
            pc.add_edge(v1, g, ()).unwrap();
            pc.add_edge(v2, g, ()).unwrap();
            pc.add_edge(g, v3, ()).unwrap();

            let back = BacktrackAlgorithm.calculate(&pc).expect("Should be valid");
            assert_eq!(9, back.len())
        }
    }

    mod test_complex_circuits {
        use super::*;
        #[test]
        fn test_purify_chain() {
            let mut pc: PureCircuitGraph = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v3 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g1 = pc.add_node(NodeUnitialised::from_gate(Gate::Purify), ());
            let v4 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v5 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g2 = pc.add_node(NodeUnitialised::from_gate(Gate::Purify), ());
            pc.add_edge(v1, g1, ()).unwrap();
            pc.add_edge(g1, v2, ()).unwrap(); // Out 1
            pc.add_edge(g1, v3, ()).unwrap();
            pc.add_edge(v3, g2, ()).unwrap();
            pc.add_edge(g2, v4, ()).unwrap(); // Out 2
            pc.add_edge(g2, v5, ()).unwrap(); // Out 3

            let back = BacktrackAlgorithm.calculate(&pc).expect("Should be valid");
            // 000
            // 001
            // 011
            // 111
            assert_eq!(7, back.len())
        }

        #[test]
        fn test_copy_purify() {
            let mut pc: PureCircuitGraph = PureCircuitGraph::<(), ()>::new();
            let v1 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v2 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let v3 = pc.add_node(NodeUnitialised::from_value(Value::Bot), ());
            let g1 = pc.add_node(NodeUnitialised::from_gate(Gate::Purify), ());
            let g2 = pc.add_node(NodeUnitialised::from_gate(Gate::Copy), ());
            pc.add_edge(v1, g1, ()).unwrap();
            pc.add_edge(g1, v2, ()).unwrap(); // Out 1
            pc.add_edge(g1, v3, ()).unwrap();
            pc.add_edge(v3, g2, ()).unwrap();
            pc.add_edge(g2, v1, ()).unwrap(); // Out 2

            let back = match BacktrackAlgorithm.calculate(&pc) {
                Ok(back) => back,
                Err(e) => panic!("{}", e.to_string()),
            };
            // 000
            // 001
            // 011
            // 111
            assert_eq!(3, back.len())
        }
    }
}
