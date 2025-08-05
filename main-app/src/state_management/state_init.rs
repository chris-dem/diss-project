use crate::{misc::cycle_enum_state, state_management::{mouse_state::EdgeManagementState, node_addition_state::ValueState}};
use bevy::{
    input::common_conditions::input_just_pressed, platform::collections::HashMap, prelude::*,
};
use pure_circuit_lib::{
    gates::{Gate, Value},
    graph::PureCircuitGraph,
};

use petgraph::prelude::*;

use super::{
    edge_management::EdgeManagementPlugin,
    mouse_state::{MousePositions, MouseState, update_mouse_resource},
    node_addition_state::GateMode,
};

#[derive(Debug, Clone, Resource, Default)]
pub struct PureCircuitResource(pub PureCircuitGraph, pub HashMap<NodeIndex, Entity>);

pub struct StateManagementPlugin;

impl Plugin for StateManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EdgeManagementPlugin)
            .init_state::<MouseState>()
            .init_state::<EdgeManagementState>()
            .init_resource::<PureCircuitResource>()
            .init_state::<GateMode>()
            .init_state::<ValueState<Value>>()
            .init_state::<ValueState<Gate>>()
            .init_resource::<MousePositions>()
            .add_systems(Update, update_mouse_resource)
            .add_systems(
                Update,
                cycle_enum_state::<MouseState>.run_if(input_just_pressed(KeyCode::BracketLeft)),
            );
    }
}
