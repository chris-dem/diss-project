use std::fmt::Display;

use crate::{
    constants::{D_RADIUS, LINE_STROKE},
    state_management::{
        events::NodeStatusUpdate, mouse_state::EdgeManagementState,
        node_addition_state::ValueComponent, state_init::PureCircuitResource,
    },
};
use bevy::{
    color::palettes::css::{ORANGE, RED, YELLOW},
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_prototype_lyon::{prelude::ShapeBuilder, prelude::*};
use petgraph::prelude::*;
use pure_circuit_lib::{EnumCycle, gates::GraphStruct};

use super::mouse_state::MouseState;

pub struct EdgeManagementPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SubStates, Hash)]
#[source(MouseState = MouseState::Edge)]
pub enum EdgeState {
    #[default]
    DefaultState,
    SelectedNode,
}

impl EnumCycle for EdgeState {
    fn toggle(&self) -> Self {
        match self {
            Self::SelectedNode => Self::DefaultState,
            Self::DefaultState => Self::SelectedNode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Resource)]
pub struct PathBuilderResource(Option<Entity>);

#[derive(Default, Resource)]
pub struct ObserverResource(Option<Entity>);

#[derive(Default, Resource, Clone, Copy, PartialEq, Eq)]
pub struct SelectedNodeMode(Option<NodeIndex>);

impl Plugin for EdgeManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PathBuilderResource>()
            .init_resource::<ObserverResource>()
            .init_resource::<SelectedNodeMode>()
            .add_sub_state::<EdgeState>()
            .add_systems(Startup, setup)
            .add_systems(OnEnter(MouseState::Edge), add_edge_detection)
            .add_systems(
                Update,
                reset_edge_mode
                    .run_if(in_state(EdgeState::SelectedNode))
                    .run_if(input_just_pressed(KeyCode::Escape)),
            )
            .add_systems(
                Update,
                reset_edge_mode
                    .run_if(in_state(MouseState::Edge))
                    .run_if(state_changed::<EdgeManagementState>),
            )
            .add_systems(
                Update,
                edge_management_toggle
                    .run_if(in_state(MouseState::Edge))
                    .run_if(input_just_pressed(KeyCode::KeyJ)),
            )
            .add_systems(
                Update,
                highlight_possible_nodes.run_if(in_state(EdgeState::SelectedNode)),
            )
            .add_systems(OnExit(MouseState::Edge), remove_edge_detection)
            .add_systems(
                Update,
                on_transition.after(TransformSystem::TransformPropagate),
            );
    }
}

fn edge_management_toggle(
    state: Res<State<EdgeManagementState>>,
    mut next_state: ResMut<NextState<EdgeManagementState>>,
) {
    next_state.set(state.get().toggle());
}

fn highlight_possible_nodes(
    selected_node_mode: Res<SelectedNodeMode>,
    edge_management_substate: Res<State<EdgeManagementState>>,
    pc_resource: Res<PureCircuitResource>,
    mut query_node: Query<(&Transform, &ValueComponent)>,
    mut gizmos: Gizmos,
) {
    let Some(indx) = selected_node_mode.0 else {
        error!("Expected node to be set");
        return;
    };
    let Some(mode) = pc_resource
        .0
        .graph
        .node_weight(indx)
        .map(GraphStruct::into_node)
    else {
        error!("Node not found");
        return;
    };
    match edge_management_substate.get() {
        EdgeManagementState::AddEdge => {
            for (t, g) in query_node {
                let Some(w) = pc_resource
                    .0
                    .graph
                    .node_weight(g.0)
                    .map(GraphStruct::into_node)
                else {
                    error!("Node not found");
                    continue;
                };
                if !w.compare_types(mode) {
                    gizmos.circle_2d(t.translation.xy(), D_RADIUS + 5., ORANGE);
                }
            }
        }
        EdgeManagementState::RemoveEdges => {
            let mut lens = query_node.transmute_lens::<&Transform>();
            for ind in pc_resource.0.get_all_neigh(indx) {
                let Some(GraphStruct {
                    additional_info: ent,
                    ..
                }) = pc_resource.0.graph.node_weight(ind)
                else {
                    error!("Index not updated");
                    continue;
                };
                let Ok(trans) = lens.query().get(*ent).copied() else {
                    error!("Entity not found in query");
                    continue;
                };
                gizmos.circle_2d(trans.translation.xy(), D_RADIUS + 5., ORANGE);
            }
        }
    }
}

fn reset_edge_mode(mut next_state: ResMut<NextState<EdgeState>>) {
    next_state.set(EdgeState::DefaultState);
}

fn setup(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = 5.;
}

fn add_edge_detection(
    query: Query<EntityRef, With<ValueComponent>>,
    mut observer_resource: ResMut<ObserverResource>,
    mut commands: Commands,
) {
    let mut observer = Observer::new(on_click);
    for e in query {
        observer.watch_entity(e.entity());
    }
    observer_resource.0 = Some(commands.spawn(observer).id());
}

fn spawn_edge(src: Vec2, dest: Vec2) -> Shape {
    let dir = (dest - src).normalize();
    let norm = Vec2 {
        x: (-dir.y),
        y: dir.x,
    };
    let neck = 10.;
    let tip = dest - (dir * (D_RADIUS + 2. + neck));
    ShapeBuilder::with(&shapes::Line(src, tip))
        .stroke(Stroke::new(RED, LINE_STROKE))
        .add(&shapes::Polygon {
            points: [tip + norm * 10., tip + norm * (-10.), tip + dir * neck].to_vec(),
            closed: true,
        })
        .build()
}

fn add_text<T: Display>(src: Vec2, dest: Vec2, val: T, text_font: TextFont) -> impl Bundle {
    (
        Text2d::new(format!("{}", val)),
        text_font,
        TextColor(Color::Srgba(YELLOW)),
        Transform {
            translation: (src + (dest - src).normalize() * (dest - src).length() / 2.).extend(10.),
            ..default()
        },
        EdgeLabel,
    )
}

#[derive(Debug, Clone, Copy, Component)]
pub struct EdgeLabel;

fn on_transition(
    query_moved_circles: Query<
        (&Transform, &ValueComponent),
        (Changed<Transform>, With<ValueComponent>, Without<EdgeLabel>),
    >,
    pc_resource: Res<PureCircuitResource>,
    query_circles: Query<&Transform, (With<ValueComponent>, Without<EdgeLabel>)>,
    mut query_text: Query<&mut Transform, With<EdgeLabel>>,
    mut query_edge: Query<(&mut Shape, &Children)>,
) {
    for (trans, ValueComponent(ind)) in query_moved_circles {
        for edge_ref in pc_resource
            .0
            .graph
            .edges_directed(*ind, Direction::Outgoing)
        {
            let (_, edge_ent) = edge_ref.weight();
            let src_trans = *trans;
            let Ok((mut edge, child)) = query_edge.get_mut(*edge_ent) else {
                error!("Edge is missing");
                continue;
            };
            let Some(dest_trans) = pc_resource
                .0
                .graph
                .node_weight(edge_ref.target())
                .and_then(|e| query_circles.get(e.additional_info).ok())
            else {
                error!("Destinaton node is missing");
                continue;
            };

            *edge = spawn_edge(src_trans.translation.xy(), dest_trans.translation.xy());
            for c in child {
                let Ok(mut trans) = query_text.get_mut(*c) else {
                    error!("Missing text position");
                    continue;
                };
                let dir = dest_trans.translation.xy() - src_trans.translation.xy();
                trans.translation =
                    (src_trans.translation.xy() + dir.normalize() * dir.length() / 2.).extend(10.);
            }
        }
        for edge_ref in pc_resource
            .0
            .graph
            .edges_directed(*ind, Direction::Incoming)
        {
            let (_, edge_ent) = edge_ref.weight();
            let dest_trans = *trans;
            let Ok((mut edge, child)) = query_edge.get_mut(*edge_ent) else {
                error!("Edge is missing");
                continue;
            };
            let Some(src_trans) = pc_resource
                .0
                .graph
                .node_weight(edge_ref.source())
                .and_then(|e| query_circles.get(e.additional_info).ok())
            else {
                error!("Destinaton node is missing");
                continue;
            };

            *edge = spawn_edge(src_trans.translation.xy(), dest_trans.translation.xy());
            for c in child {
                let Ok(mut trans) = query_text.get_mut(*c) else {
                    error!("Missing text position");
                    continue;
                };
                let dir = dest_trans.translation.xy() - src_trans.translation.xy();
                trans.translation =
                    (src_trans.translation.xy() + dir.normalize() * dir.length() / 2.).extend(10.);
            }
        }
    }
}

fn on_click(
    trigger: Trigger<Pointer<Click>>,
    query: Query<&ValueComponent>,
    mouse_state: Res<State<EdgeState>>,
    edge_management_state: Res<State<EdgeManagementState>>,
    query_locs: Query<&Transform>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut next_mouse_state: ResMut<NextState<EdgeState>>,
    mut path_builder: ResMut<PathBuilderResource>,
    mut selected_node: ResMut<SelectedNodeMode>,
    mut pc_resource: ResMut<PureCircuitResource>,
    mut event_writer_nod: EventWriter<NodeStatusUpdate>,
) {
    let Ok(graph_node) = query.get(trigger.target()) else {
        error!("Query does not contain a graph node");
        return;
    };

    match **mouse_state {
        EdgeState::DefaultState => {
            path_builder.0 = Some(trigger.target());
            selected_node.0 = Some(graph_node.0);
            next_mouse_state.set(mouse_state.toggle());
        }
        EdgeState::SelectedNode => {
            let Some(ind) = selected_node.0 else {
                error!("No selected node");
                return;
            };
            let Some(GraphStruct {
                node: sel_gate_mode,
                additional_info: src_ent,
            }) = pc_resource.0.graph.node_weight(ind).copied()
            else {
                error!("No selected node found");
                return;
            };
            let Some(GraphStruct {
                node: current_node,
                additional_info: dest_ent,
            }) = pc_resource.0.graph.node_weight(graph_node.0).copied()
            else {
                error!("Current node not in graph");
                return;
            };

            if current_node.compare_types(sel_gate_mode) {
                warn!("Selected homogeneous node");
                return;
            }

            let Ok([src_trans, dest_trans]) = query_locs
                .get_many([src_ent, dest_ent])
                .map(|x| [x[0].translation.xy(), x[1].translation.xy()])
            else {
                error!("Missing locations");
                return;
            };

            let font = asset_server.load("fonts/FiraSans-Bold.ttf");
            let text_font = TextFont {
                font: font.clone(),
                font_size: 35.,
                ..default()
            };

            let gate_indx = match **edge_management_state {
                EdgeManagementState::AddEdge => {
                    let edge_entity = commands.spawn(spawn_edge(src_trans, dest_trans)).id();
                    match pc_resource.0.add_edge(ind, graph_node.0, edge_entity) {
                        Ok((node_ind, _, val)) => {
                            commands.entity(edge_entity).with_children(|parent| {
                                parent.spawn(add_text(src_trans, dest_trans, val, text_font));
                            });
                            // event_writer_add.write(EdgeAdditionEvent(e_indx));
                            node_ind
                        }
                        Err(e) => {
                            error!("Error adding edge {e:?}");
                            return;
                        }
                    }
                }
                EdgeManagementState::RemoveEdges => {
                    match pc_resource.0.remove_edge(ind, graph_node.0) {
                        Ok((nod_indx, edges)) => {
                            for (_, ent) in edges {
                                commands.entity(ent).despawn();
                            }
                            nod_indx
                        }
                        Err(e) => {
                            error!("Error adding edge {e:?}");
                            return;
                        }
                    }
                }
            };
            event_writer_nod.write(NodeStatusUpdate(gate_indx));
            selected_node.0 = None;
            next_mouse_state.set(mouse_state.toggle());
        }
    };
}

fn remove_edge_detection(mut observer_resource: ResMut<ObserverResource>, mut commands: Commands) {
    if let Some(obs) = observer_resource.0.take() {
        commands.entity(obs).despawn();
    }
}
