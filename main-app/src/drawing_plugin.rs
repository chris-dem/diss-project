use bevy::{
    color::palettes::css::YELLOW,
    ecs::{observer::TriggerTargets, relationship::RelatedSpawnerCommands},
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use pure_circuit_lib::gates::{Gate, Value};
use pure_circuit_lib::{EnumCycle, gates::NewNode};

use crate::{
    assets::{ASSET_DICT, generate_bundle_from_asset},
    constants::D_RADIUS,
    state_management::{
        mouse_state::{MousePositions, MouseState},
        node_addition_state::{GateMode, GraphNode, ValueComponent, ValueState},
    },
};
pub struct DrawingPlugin;

#[derive(Component, Debug, Clone, Copy)]
pub struct MouseCircle;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_svg::prelude::SvgPlugin)
            .add_systems(
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
    value_state: Res<State<ValueState<Value>>>,
    gate_state: Res<State<ValueState<Gate>>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(pos) = mouse_resource.0 else {
        return;
    };
    let val = match **gate_mode {
        GateMode::Gate => NewNode::GateNode(gate_state.0),
        GateMode::Value => NewNode::ValueNode(value_state.0),
    };

    let mut entity = commands.spawn((
        ShapeBuilder::with(&shapes::Circle {
            center: Vec2::splat(0.),
            radius: D_RADIUS,
        })
        .fill(gate_mode.get_col())
        .build(),
        GraphNode(**gate_mode),
        ValueComponent(val),
        Pickable::default(),
        Transform {
            translation: pos.extend(0.),
            ..default()
        },
    ));
    entity.with_children(|parent| value_spawner(parent, val, asset_server));

    entity
        .observe(on_drag)
        .observe(on_hover_enter)
        .observe(on_click)
        .observe(on_hover_exit);
}

fn on_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut Children, Entity, &mut ValueComponent), With<ValueComponent>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok((children, entity, ref mut current_value)) = query.get_mut(trigger.target) else {
        warn!("Element not found");
        return;
    };

    for entity in children.entities() {
        commands.entity(entity).despawn();
    }

    current_value.0 = match current_value.0 {
        NewNode::GateNode(b) => NewNode::GateNode(b.toggle()),
        NewNode::ValueNode(b) => NewNode::ValueNode(b.toggle()),
    };

    commands
        .entity(entity)
        .with_children(|parent| value_spawner(parent, current_value.0, asset_server));
}

fn value_spawner(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    value: NewNode,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 35.,
        ..default()
    };

    match value {
        NewNode::ValueNode(val) => {
            let ind = val as usize;
            for bund in generate_bundle_from_asset(
                ASSET_DICT[ind].0.as_slice(),
                ASSET_DICT[ind].1.as_slice(),
                ASSET_DICT[ind].2,
            )
            .iter()
            .cloned()
            {
                parent.spawn(bund);
            }
        }
        NewNode::GateNode(val) => {
            parent.spawn((
                Text2d::new(format!("{}", val)),
                text_font,
                TextColor(Color::Srgba(YELLOW)),
                Transform {
                    translation: Vec2::splat(0.).extend(5.),
                    ..default()
                },
            ));
        }
    }
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
