use crate::constants::D_RADIUS;
use bevy::{
    color::palettes::css::{ORANGE, RED},
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use pure_circuit_lib::EnumCycle;

use super::{
    mouse_state::MouseState,
    node_addition_state::{GateMode, GraphNode},
};

pub struct EdgeManagementPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SubStates, Hash)]
#[source(MouseState = MouseState::Edge)]
pub enum EdgeState {
    #[default]
    DefaultState,
    SelectedNode,
}

impl EdgeState {
    fn toggle_state(&self) -> Self {
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
pub struct SelectedNodeMode(Option<GateMode>);

type EdgePair = (Entity, Entity);

#[derive(Default, Resource)]
pub struct EdgeListResource(pub Vec<EdgePair>);

impl Plugin for EdgeManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PathBuilderResource>()
            .init_resource::<ObserverResource>()
            .init_resource::<EdgeListResource>()
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
                highlight_possible_nodes.run_if(in_state(EdgeState::SelectedNode)),
            )
            .add_systems(OnExit(MouseState::Edge), remove_edge_detection)
            .add_systems(FixedPostUpdate, draw_edges);
    }
}

fn highlight_possible_nodes(
    selected_node_mode: Res<SelectedNodeMode>,
    query: Query<(&Transform, &GraphNode)>,
    mut gizmos: Gizmos,
) {
    let Some(mode) = selected_node_mode.0 else {
        error!("Expected node to be set");
        return;
    };

    for (t, g) in query {
        if g.0 == mode {
            gizmos.circle_2d(t.translation.xy(), D_RADIUS + 5., ORANGE);
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

fn draw_edges(mut gizmos: Gizmos, query: Query<&Transform>, edge_list: Res<EdgeListResource>) {
    for (s, t) in edge_list.0.iter() {
        let Ok([start, end]) = query.get_many([*s, *t]) else {
            error!("Cannot find tuple");
            continue;
        };
        let start = start.translation;
        let end = end.translation;
        let offset = (end - start).normalize_or_zero() * D_RADIUS;
        gizmos.arrow(start + offset, end - offset, RED);
    }
}

fn add_edge_detection(
    query: Query<EntityRef, With<GraphNode>>,
    mut observer_resource: ResMut<ObserverResource>,
    mut commands: Commands,
) {
    let mut observer = Observer::new(on_click);
    for e in query {
        observer.watch_entity(e.entity());
    }
    observer_resource.0 = Some(commands.spawn(observer).id());
}

fn on_click(
    trigger: Trigger<Pointer<Click>>,
    query: Query<&GraphNode>,
    mouse_state: Res<State<EdgeState>>,
    mut next_mouse_state: ResMut<NextState<EdgeState>>,
    mut path_builder: ResMut<PathBuilderResource>,
    mut selected_node: ResMut<SelectedNodeMode>,
    mut edge_list: ResMut<EdgeListResource>,
) {
    let Ok(graph_node) = query.get(trigger.target()) else {
        error!("Query does not contain a graph node");
        return;
    };
    match **mouse_state {
        EdgeState::DefaultState => {
            path_builder.0 = Some(trigger.target());
            selected_node.0 = Some(graph_node.0.toggle());
            next_mouse_state.set(mouse_state.toggle_state());
        }
        EdgeState::SelectedNode => {
            let Some(sel_gate_mode) = selected_node.0 else {
                error!("No selected node found");
                return;
            };
            if graph_node.0 != sel_gate_mode {
                info!("Selected opposite node");
                return;
            }
            let Some(source) = path_builder.0 else {
                error!("Invalid configuration");
                next_mouse_state.set(mouse_state.toggle_state());
                return;
            };
            edge_list.0.push((source, trigger.target()));
            selected_node.0 = None;
            path_builder.0 = None;
            next_mouse_state.set(mouse_state.toggle_state());
        }
    };
}

fn remove_edge_detection(mut observer_resource: ResMut<ObserverResource>, mut commands: Commands) {
    if let Some(obs) = observer_resource.0.take() {
        commands.entity(obs).despawn();
    }
}
