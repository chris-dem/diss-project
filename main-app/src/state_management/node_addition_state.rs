use std::fmt::Display;

use bevy::prelude::*;

use crate::constants::{GATETEXT, GCOLOUR, VALTEXT, VCOLOUR};

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, States)]
pub enum GateMode {
    #[default]
    Value,
    Gate,
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, Component)]
pub struct Interactable;

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, Component)]
pub struct GraphNode;

impl Display for GateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gate => write!(f, "{}", GATETEXT),
            Self::Value => write!(f, "{}", VALTEXT),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct GateCircle;

#[derive(Component, Debug, Clone, Copy)]
pub struct ValueCircle;

impl GateMode {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Value => Self::Gate,
            Self::Gate => Self::Value,
        }
    }

    pub fn get_col(&self) -> Color {
        match self {
            Self::Value => VCOLOUR,
            Self::Gate => GCOLOUR,
        }
    }
}
