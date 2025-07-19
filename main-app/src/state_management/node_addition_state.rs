use crate::constants::{GATETEXT, GCOLOUR, VALTEXT, VCOLOUR};
use bevy::{prelude::*, state::state::FreelyMutableState};
use pure_circuit_lib::{
	EnumCycle,
	gates::{Gate, Value},
};
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, States, EnumCycle)]
pub enum GateMode {
	#[default]
	Value,
	Gate,
}

pub trait ValueStateTraits: Debug + Clone + Copy + Default + Hash + PartialEq + Eq{}

// Blanket implementation for any type that meets the requirements
impl<T> ValueStateTraits for T where
	T: Debug + Clone + Copy + Default + Hash + PartialEq + Eq
{
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct ValueState<T: ValueStateTraits>(pub T);


impl FreelyMutableState for ValueState<Value> {} 
impl States for ValueState<Value> {} 
impl FreelyMutableState for ValueState<Gate> {} 
impl States for ValueState<Gate> {} 



impl<T: ValueStateTraits + EnumCycle> EnumCycle for ValueState<T> {
	fn toggle(&self) -> Self {
		Self(self.0.toggle())
	}
}


impl<T: ValueStateTraits> Display for ValueState<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} value", self)
	}
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Component)]
pub struct ValueComponent(pub NodeValue);

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
