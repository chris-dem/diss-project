use bevy::{
    color::palettes::css::YELLOW,
    input::common_conditions::{input_just_pressed, input_pressed},
    picking::prelude::*,
    prelude::*,
};

use bevy_prototype_lyon::prelude::*;

use crate::{
    constants::D_RADIUS,
    state_management::{
        mouse_state::{MousePositions, MouseState},
        node_addition_state::{GateMode, GraphNode},
    },
};

pub struct DrawingPlugin;

#[derive(Component, Debug, Clone, Copy)]
pub struct MouseCircle;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            hover_draw
                .run_if(in_state(MouseState::Node))
                .run_if(input_pressed(KeyCode::KeyA))
                .after(TransformSystem::TransformPropagate),
        )
        .add_systems(
            Update,
            click_draw
                .run_if(in_state(MouseState::Node))
                .run_if(input_pressed(KeyCode::KeyA))
                .run_if(input_just_pressed(MouseButton::Left)),
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

fn click_draw(
    mouse_resource: Res<MousePositions>,
    gate_mode: Res<State<GateMode>>,
    mut commands: Commands,
) {
    let Some(pos) = mouse_resource.0 else {
        return;
    };

    let mut entity = commands.spawn((
        ShapeBuilder::with(&shapes::Circle {
            center: Vec2::splat(0.),
            radius: D_RADIUS,
        })
        .fill(gate_mode.get_col())
        .build(),
        GraphNode(**gate_mode),
        Pickable::default(),
        Transform {
            translation: pos.extend(0.),
            ..default()
        },
    ));

    entity
        .observe(on_drag)
        .observe(on_hover_enter)
        .observe(on_hover_exit);
}

fn on_drag(
    trigger: Trigger<Pointer<Drag>>,
    mut query: Query<&mut Transform, With<Pickable>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if !key.pressed(KeyCode::KeyM) {
        return;
    }

    if let Ok(ref mut pos) = query.get_mut(trigger.target()) {
        pos.translation.x += trigger.delta.x;
        pos.translation.y -= trigger.delta.y;
    }
}

fn on_hover_enter(trigger: Trigger<Pointer<Over>>, mut query: Query<&mut Shape, With<Pickable>>) {
    let Ok(ref mut pos) = query.get_mut(trigger.target) else {
        return;
    };
    pos.stroke = Some(Stroke::new(Color::from(YELLOW), 5.))
}

fn on_hover_exit(trigger: Trigger<Pointer<Out>>, mut query: Query<&mut Shape, With<Pickable>>) {
    let Ok(ref mut pos) = query.get_mut(trigger.target) else {
        return;
    };
    pos.stroke = None;
}
