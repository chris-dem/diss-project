use bevy::{prelude::*, state::state::FreelyMutableState};
use pure_circuit_lib::EnumCycle;

pub fn cycle_enum_state<T: EnumCycle + States + FreelyMutableState>(
    current_state: Res<State<T>>,
    mut next_state: ResMut<NextState<T>>,
) {
    next_state.set(current_state.toggle());
}

