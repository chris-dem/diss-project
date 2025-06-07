use bevy::prelude::*;


// Current State of the mouse
#[derive(Debug, Clone, Copy, Default, States, PartialEq, Eq, Hash)]
pub enum MouseState {
    // Add nodes
    #[default]
    Node,
    // Navigate the map
    Hover,
    // Add edges
    Edge,
}



