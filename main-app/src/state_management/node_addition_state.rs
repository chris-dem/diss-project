use bevy::prelude::*;


#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, States)]
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
