use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::{
    edge_management::EdgeManagementPlugin,
    mouse_state::{MousePositions, MouseState, cycle_mouse_state, update_mouse_resource},
    node_addition_state::GateMode,
};

pub struct StateManagementPlugin;

impl Plugin for StateManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EdgeManagementPlugin)
            .init_state::<MouseState>()
            .init_state::<GateMode>()
            .init_resource::<MousePositions>()
            .add_systems(Update, update_mouse_resource)
            .add_systems(
                Update,
                cycle_mouse_state.run_if(input_just_pressed(KeyCode::BracketLeft)),
            );
    }
}
