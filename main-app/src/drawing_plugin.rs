use bevy::{
    color::palettes::css::YELLOW,
    ecs::{observer::TriggerTargets, relationship::RelatedSpawnerCommands},
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use pure_circuit_lib::gates::{Gate, Value};
use pure_circuit_lib::{
    EnumCycle,
    gates::{GraphNode, NewNode, NodeUnitialised},
};

use crate::{
    assets::{ASSET_DICT, generate_bundle_from_asset},
    constants::D_RADIUS,
    state_management::{
        mouse_state::{MousePositions, MouseState},
        node_addition_state::{GateMode, ValueComponent, ValueState},
        state_init::PureCircuitResource,
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
    mut pc_resource: ResMut<PureCircuitResource>,
    mut commands: Commands,
) {
    let Some(pos) = mouse_resource.0 else {
        return;
    };
    let val = match **gate_mode {
        GateMode::Gate => NodeUnitialised::from_gate(gate_state.0),
        GateMode::Value => NodeUnitialised::from_value(value_state.0),
    };
    let index = pc_resource.0.add_node(val);

    let mut entity = commands.spawn((
        ShapeBuilder::with(&shapes::Circle {
            center: Vec2::splat(0.),
            radius: D_RADIUS,
        })
        .fill(gate_mode.get_col())
        .build(),
        ValueComponent(index),
        Pickable::default(),
        Transform {
            translation: pos.extend(0.),
            ..default()
        },
    ));
    pc_resource.1.insert(index, entity.id());
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
    mut pc_resource: ResMut<PureCircuitResource>,
    asset_server: Res<AssetServer>,
) {
    let Ok((children, entity, ref mut current_value)) = query.get_mut(trigger.target) else {
        warn!("Element not found");
        return;
    };

    for entity in children.entities() {
        commands.entity(entity).despawn();
    }

    let node = match pc_resource.0.graph.node_weight(current_value.0).copied() {
        Some(GraphNode::GateNode { gate, .. }) => NodeUnitialised::GateNode {
            gate: gate.toggle(),
            state_type: NewNode,
        },
        Some(GraphNode::ValueNode(b)) => NodeUnitialised::ValueNode(b.toggle()),
        _ => {
            error!("Node does not exist");
            return;
        }
    };

    if let Err(err) = pc_resource.0.update_node(current_value.0, node) {
        error!("Error updating node. Error {err:?}");
        return;
    }

    commands
        .entity(entity)
        .with_children(|parent| value_spawner(parent, node, asset_server));
}

fn value_spawner(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    value: NodeUnitialised,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 35.,
        ..default()
    };

    match value {
        NodeUnitialised::ValueNode(val) => {
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
        NodeUnitialised::GateNode { gate: val, .. } => {
            // TODO
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
