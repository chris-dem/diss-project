use bevy::{
    color::palettes::css::YELLOW, input::common_conditions::input_just_pressed,
    picking::prelude::*, prelude::*,
};

use bevy_prototype_lyon::prelude::*;

use crate::{
    constants::{D_RADIUS, VCOLOUR},
    state_management::{
        mouse_state::{MousePositions, MouseState},
        node_addition_state::{GateCircle, GateMode, ValueCircle},
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

    let mut entity = match **gate_mode {
        GateMode::Value => commands.spawn((
            ShapeBuilder::with(&shapes::Circle {
                center: pos,
                radius: D_RADIUS,
            })
            .fill(VCOLOUR)
            .build(),
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
    entity
        .observe(on_drag)
        .observe(on_hover_enter)
        .observe(on_hover_exit);
}

fn on_drag(trigger: Trigger<Pointer<Drag>>, mut query: Query<&mut Transform, With<Pickable>>) {
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
