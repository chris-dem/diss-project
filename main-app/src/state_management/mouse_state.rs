use bevy::prelude::*;
use pure_circuit_lib::EnumCycle;
use std::{fmt::Display, hash::Hash};

// Current State of the mouse
#[derive(Debug, Clone, Copy, Default, States, PartialEq, Eq, Hash, EnumCycle)]
pub enum MouseState {
    // Add nodes
    #[default]
    Node,
    // Add edges
    Edge,
}

impl Display for MouseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node => write!(f, "Node Mode"),
            Self::Edge => write!(f, "Edge Mode"),
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MousePositions(pub Option<Vec2>);

pub fn update_mouse_resource(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    mut position_resource: ResMut<MousePositions>,
) {
    let (camera, camera_transform) = *camera_query;
    let window = window.single().ok();
    let position: Option<Vec2> = window
        .and_then(Window::cursor_position)
        .and_then(|c| camera.viewport_to_world_2d(camera_transform, c).ok());

    position_resource.0 = position;
}
