use bevy::{
    color::palettes::css::YELLOW, input::common_conditions::input_just_pressed,
    picking::prelude::*, prelude::*,
};

use crate::{
    constants::D_RADIUS,
    state_management::{
        mouse_state::{MousePositions, MouseState},
        node_addition_state::{GateCircle, GateMode, Interactable, ValueCircle},
    },
};

pub struct DrawingPlugin;

#[derive(Component, Debug, Clone, Copy)]
pub struct MouseCircle;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MouseState::Node), draw_setup)
            .add_systems(
                PostUpdate,
                hover_draw
                    .run_if(in_state(MouseState::Node))
                    .after(TransformSystem::TransformPropagate),
            )
            .add_systems(
                Update,
                click_draw
                    .run_if(in_state(MouseState::Node))
                    .run_if(input_just_pressed(KeyCode::Enter)),
            );
    }
}

fn hover_draw(
    mouse_resource: Res<MousePositions>,
    gate_mode: Res<State<GateMode>>,
    mut gizmos: Gizmos,
) {
    let Some(world_pos) = mouse_resource.0 else {
        return;
    };

    let col = gate_mode.get_col();

    // Should be the same as world_pos
    gizmos.circle_2d(world_pos, D_RADIUS, col);
}

fn draw_setup() {}

fn click_draw(
    mouse_resource: Res<MousePositions>,
    gate_mode: Res<State<GateMode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let Some(pos) = mouse_resource.0 else {
        return;
    };
    let col_handle = material.add(gate_mode.get_col());
    let circle_handle = meshes.add(Circle::new(D_RADIUS));
    let mut entitity = match **gate_mode {
        GateMode::Value => commands.spawn((
            Mesh2d(circle_handle),
            MeshMaterial2d(col_handle),
            Transform {
                translation: pos.extend(0.),
                ..Transform::default()
            },
            Outline {
                width: Val::Px(0.5),
                offset: Val::Px(0.5),
                color: Color::from(YELLOW),
            },
            ValueCircle,
            Pickable::default(),
        )),
        GateMode::Gate => commands.spawn((
            Mesh2d(circle_handle),
            MeshMaterial2d(col_handle),
            Transform {
                translation: pos.extend(0.),
                ..Transform::default()
            },
            GateCircle,
            Outline {
                width: Val::Px(10.),
                offset: Val::Px(0.5),
                color: Color::from(YELLOW),
            },
            Pickable::default(),
        )),
    };

    entitity.observe(on_drag);
}

fn on_drag(trigger: Trigger<Pointer<Drag>>, mut query: Query<&mut Transform, With<Pickable>>) {
    if let Ok(ref mut pos) = query.get_mut(trigger.target()) {
        pos.translation.x += trigger.delta.x;
        pos.translation.y -= trigger.delta.y;
    }
}
//
// fn on_hover_enter(
//     trigger: Trigger<Pointer<Drag>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut material: ResMut<Assets<ColorMaterial>>,
//     mut commands: Commands,
//     mut query: Query<&mut Transform, With<Pickable>>,
// ) {
//     let Ok(current_pos) = query.get_mut(trigger.target) else {
//         return;
//     };
//
//     let circle = Outline
//
//     commands.spawn()
// }
//
// fn on_hover_eit(trigger: Trigger<Pointer<Drag>>, mut query: Query<&mut Transform, With<Pickable>>) {
//     if let Ok(ref mut pos) = query.get_mut(trigger.target()) {
//         pos.translation.x += trigger.delta.x;
//         pos.translation.y -= trigger.delta.y;
//     }
// }
