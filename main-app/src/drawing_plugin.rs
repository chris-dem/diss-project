use bevy::{
    color::palettes::{
        css::YELLOW,
        tailwind::{RED_200, RED_500},
    },
    ecs::relationship::RelatedSpawnerCommands,
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use itertools::Itertools;
use petgraph::Direction;
use pure_circuit_lib::gates::{Gate, GraphStruct, Value};
use pure_circuit_lib::{
    EnumCycle,
    gates::{GateStatus, GraphNode, NewNode, NodeUnitialised, NodeValue},
};

use crate::{
    assets::{ASSET_DICT, generate_bundle_from_asset},
    constants::D_RADIUS,
    state_management::{
        events::{IndexReset, NodeStatusUpdate, NodeUpdate, SolutionReset},
        mouse_state::{MousePositions, MouseState},
        node_addition_state::{GateMode, ValueComponent, ValueState},
        state_init::PureCircuitResource,
    },
};
pub struct DrawingPlugin;

#[derive(Component, Debug, Clone, Copy)]
pub struct MouseCircle;

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub(crate) struct GateStatusComponent(pub(crate) GateStatus);

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredNode>()
            .add_plugins(bevy_svg::prelude::SvgPlugin)
            .add_systems(
                Update,
                on_hover_del
                    .run_if(input_just_pressed(KeyCode::KeyD))
                    .run_if(not(resource_equals(HoveredNode(None))))
                    .run_if(not(in_state(MouseState::Edge))),
            )
            .add_systems(PostUpdate, highlight_error_values)
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

fn highlight_error_values(
    query: Query<(Entity, &Children, &GateStatusComponent), Changed<GateStatusComponent>>,
    query_err: Query<Entity, With<ErrorCircle>>,
    mut commands: Commands,
) {
    for (ent, children, status_component) in query {
        for child in children {
            if let Ok(child) = query_err.get(*child) {
                commands.entity(child).despawn();
            }
        }
        match status_component.0 {
            GateStatus::Valid => (),
            status => {
                info!("Adding error circles {status_component:?}");
                commands.entity(ent).with_child(spawn_error_circle(status));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub(crate) struct ErrorCircle;

fn spawn_error_circle(status: GateStatus) -> impl Bundle {
    let col = match status {
        GateStatus::InvalidArity => Color::Srgba(RED_200),
        GateStatus::InvalidValues => Color::Srgba(RED_500),
        _ => panic!("Gate status cannot be valid"),
    };
    (
        ShapeBuilder::with(&shapes::Circle {
            center: Vec2::splat(0.),
            radius: D_RADIUS + 10.,
        })
        .stroke(Stroke::new(col, 5.))
        .build(),
        ErrorCircle,
    )
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

#[allow(clippy::too_many_arguments)]
fn click_draw(
    mouse_resource: Res<MousePositions>,
    gate_mode: Res<State<GateMode>>,
    value_state: Res<State<ValueState<Value>>>,
    gate_state: Res<State<ValueState<Gate>>>,
    mut pc_resource: ResMut<PureCircuitResource>,
    mut event_writer_status: EventWriter<NodeUpdate>,
    mut event_sol_reset: EventWriter<SolutionReset>,
    mut commands: Commands,
) {
    let Some(pos) = mouse_resource.0 else {
        return;
    };
    let val = match **gate_mode {
        GateMode::Gate => NodeUnitialised::from_gate(gate_state.0),
        GateMode::Value => NodeUnitialised::from_value(value_state.0),
    };

    let mut entity = commands.spawn((
        ShapeBuilder::with(&shapes::Circle {
            center: Vec2::splat(0.),
            radius: D_RADIUS,
        })
        .fill(gate_mode.get_col())
        .build(),
        Pickable::default(),
        Transform {
            translation: pos.extend(10.),
            ..default()
        },
    ));
    let index = pc_resource.0.add_node(val, entity.id());
    entity.insert(ValueComponent(index));
    event_sol_reset.write(SolutionReset);
    event_writer_status.write(NodeUpdate(index));

    if *gate_mode.get() == GateMode::Gate {
        let Some(NodeValue::GateNode {
            gate: _,
            state_type,
        }) = pc_resource
            .0
            .graph
            .node_weight(index)
            .map(GraphStruct::into_node)
        else {
            error!("Node should exist");
            return;
        };
        entity.insert(GateStatusComponent(state_type));
    }

    entity
        .observe(on_drag)
        .observe(on_hover_enter)
        .observe(on_click)
        .observe(on_hover_exit);
}

#[allow(clippy::too_many_arguments)]
fn on_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<&mut ValueComponent, With<ValueComponent>>,
    mut pc_resource: ResMut<PureCircuitResource>,
    mut event_writer: EventWriter<NodeStatusUpdate>,
    mut event_writer_status: EventWriter<NodeUpdate>,
    mut event_sol_reset: EventWriter<SolutionReset>,
    mut event_idx_reset: EventWriter<IndexReset>,
    mouse_state: Res<State<MouseState>>,
) {
    if *mouse_state.get() == MouseState::Edge {
        return;
    }

    let Ok(ref mut current_value) = query.get_mut(trigger.target) else {
        warn!("Element not found");
        return;
    };

    let node = match pc_resource
        .0
        .graph
        .node_weight(current_value.0)
        .map(GraphStruct::into_node)
    {
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

    let gates = match pc_resource.0.update_node(current_value.0, node) {
        Ok(many) => many,
        Err(err) => {
            error!("Error updating node. Error {err:?}");
            return;
        }
    };

    event_writer_status.write(NodeUpdate(current_value.0));
    event_writer.write_batch(gates.into_iter().map(NodeStatusUpdate));
    if matches!(node, NodeValue::GateNode { .. }) {
        event_sol_reset.write_default();
    } else {
        event_idx_reset.write_default();
    }
}

#[derive(Debug, Clone, Resource, Default, PartialEq, Eq)]
struct HoveredNode(Option<Entity>);

pub(crate) fn value_spawner(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    value: NodeUnitialised,
    asset_server: &AssetServer,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 35.,
        ..default()
    };

    match value {
        NodeUnitialised::ValueNode(val) => {
            let ind: usize = val.into();
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
    mouse_state: Res<State<MouseState>>,
    mut query: Query<&mut Transform, With<Pickable>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if !key.pressed(KeyCode::KeyM) && !matches!(mouse_state.get(), MouseState::Node) {
        return;
    }

    if let Ok(ref mut pos) = query.get_mut(trigger.target()) {
        pos.translation.x += trigger.delta.x;
        pos.translation.y -= trigger.delta.y;
    }
}

fn on_hover_del(
    key: Res<ButtonInput<KeyCode>>,
    query_indx: Query<&ValueComponent>,
    mut hovered_node: ResMut<HoveredNode>,
    mut pc_resource: ResMut<PureCircuitResource>,
    mut event_writer: EventWriter<NodeStatusUpdate>,
    mut event_sol_writer: EventWriter<SolutionReset>,
    mut commands: Commands,
) {
    let Some(target) = hovered_node.0.take() else {
        error!("Entity should be stored");
        return;
    };
    let Ok(ValueComponent(indx)) = query_indx.get(target) else {
        error!("Node with out index");
        return;
    };
    if key.just_pressed(KeyCode::KeyD) {
        commands.entity(target).despawn();
        let edges = pc_resource
            .0
            .graph
            .edges_directed(*indx, Direction::Incoming)
            .chain(
                pc_resource
                    .0
                    .graph
                    .edges_directed(*indx, Direction::Outgoing),
            )
            .map(|e| e.weight().1)
            .collect_vec();
        match pc_resource.0.remove_node(*indx) {
            Err(e) => error!("Error {e} for when deleting node"),
            Ok(ind) => {
                for e in edges {
                    commands.entity(e).despawn();
                }
                for n in ind {
                    event_writer.write(NodeStatusUpdate(n));
                }
            }
        };
        event_sol_writer.write_default();
    }
}

fn on_hover_enter(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut Shape, With<Pickable>>,
    mut hovered_node: ResMut<HoveredNode>,
) {
    let Ok(ref mut pos) = query.get_mut(trigger.target) else {
        return;
    };
    hovered_node.0 = Some(trigger.target);
    pos.stroke = Some(Stroke::new(Color::from(YELLOW), 5.));
}

fn on_hover_exit(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut Shape, With<Pickable>>,
    mut hovered_node: ResMut<HoveredNode>,
) {
    let Ok(ref mut pos) = query.get_mut(trigger.target) else {
        return;
    };
    hovered_node.0 = None;
    pos.stroke = None;
}
