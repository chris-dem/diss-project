use bevy::prelude::*;
use std::{fmt::Display, hash::Hash};

// Current State of the mouse
#[derive(Debug, Clone, Copy, Default, States, PartialEq, Eq, Hash)]
pub enum MouseState {
    // Add nodes
    #[default]
    Node,
    // Add edges
    Edge,
    // Navigate the map
    Hover,
}

impl Display for MouseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node => write!(f, "Node Mode"),
            Self::Edge => write!(f, "Edge Mode"),
            Self::Hover => write!(f, "Hover Mode"),
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MousePositions(pub Option<Vec2>);

impl MouseState {
    pub fn cycle_states(&self) -> Self {
        match self {
            Self::Node => Self::Edge,
            Self::Edge => Self::Hover,
            Self::Hover => Self::Node,
        }
    }
}

pub fn cycle_mouse_state(
    mouse_state: Res<State<MouseState>>,
    mut updated_state: ResMut<NextState<MouseState>>,
) {
    updated_state.set(mouse_state.cycle_states());
}

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
