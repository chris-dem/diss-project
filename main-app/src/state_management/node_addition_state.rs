use crate::constants::{GATETEXT, GCOLOUR, VALTEXT, VCOLOUR};
use bevy::prelude::*;
use pure_circuit_lib::{EnumCycle, gates::Value};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, States, EnumCycle)]
pub enum GateMode {
    #[default]
    Value,
    Gate,
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, States, EnumCycle)]
pub enum ValueState {
    #[default]
    Bot,
    Zero,
    One,
}

impl Display for ValueState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} value", self)
    }
}

impl Into<u8> for ValueState {
    fn into(self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Bot => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, Component)]
pub struct ValueComponent(pub Value);

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, Component)]
pub struct Interactable;

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, Component)]
pub struct GraphNode(pub GateMode);

impl Display for GateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gate => write!(f, "{}", GATETEXT),
            Self::Value => write!(f, "{}", VALTEXT),
        }
    }
}

impl GateMode {
    pub fn get_col(&self) -> Color {
        match self {
            Self::Value => VCOLOUR,
            Self::Gate => GCOLOUR,
        }
    }
}
