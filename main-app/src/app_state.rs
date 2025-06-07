use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct AppState {
    pub mode: GateMode,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum GateMode {
    #[default]
    Value,
    Gate,
}

impl GateMode {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Value => Self::Gate,
            Self::Gate => Self::Value,
        }
    }
}

pub fn toggle_state(mut state: ResMut<AppState>) {
    state.mode = state.mode.toggle();
}
