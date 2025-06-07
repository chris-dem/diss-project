use bevy::prelude::*;

use super::{mouse_state::MouseState, node_addition_state::GateMode};

pub struct StateManagementPlugin;

impl Plugin for StateManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MouseState>().init_state::<GateMode>();
    }
}
