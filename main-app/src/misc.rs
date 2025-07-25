use bevy::{prelude::*, state::state::FreelyMutableState};
use pure_circuit_lib::{EnumCycle, gates::GraphNode};

use crate::state_management::node_addition_state::GateMode;

pub fn cycle_enum_state<T: EnumCycle + States + FreelyMutableState>(
    current_state: Res<State<T>>,
    mut next_state: ResMut<NextState<T>>,
) {
    next_state.set(current_state.toggle());
}

pub fn compare_nodes(current: GraphNode, other: GateMode) -> bool {
    match (current, other) {
        (GraphNode::ValueNode(_), GateMode::Value) => true,
        (GraphNode::GateNode { .. }, GateMode::Gate) => true,
        _ => false,
    }
}
