use bevy::prelude::*;
use petgraph::prelude::*;
use pure_circuit_lib::gates::{GraphStruct, NodeValue};

use crate::{
    algo_execution::back::{SolutionIndex, SolutionSet},
    drawing_plugin::{ErrorCircle, GateStatusComponent, value_spawner},
    state_management::{node_addition_state::ValueComponent, state_init::PureCircuitResource},
};

#[derive(Debug, Clone, Event)]
pub struct NodeStatusUpdate(pub NodeIndex);

#[derive(Debug, Clone, Event)]
pub struct NodeUpdate(pub NodeIndex);

pub struct EventManagerPlugin;

#[derive(Debug, Clone, Event)]
pub struct ButtonEvoEvent;

#[derive(Debug, Clone, Event)]
pub struct ButtonHillEvent;

#[derive(Debug, Clone, Event)]
pub struct BacktrackEvent;

#[derive(Debug, Clone, Event, Default)]
pub struct SolutionReset;

#[derive(Debug, Clone, Event, Default)]
pub struct IndexReset;

impl Plugin for EventManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, manage_node_update_status)
            .add_systems(Update, manage_node_update)
            .add_systems(Update, manage_solution_reset)
            .add_systems(Update, index_reset)
            .add_event::<NodeUpdate>()
            .add_event::<ButtonEvoEvent>()
            .add_event::<ButtonHillEvent>()
            .add_event::<BacktrackEvent>()
            .add_event::<SolutionReset>()
            .add_event::<IndexReset>()
            .add_event::<NodeStatusUpdate>();
    }
}

/// Update status of gate with correct error circle or remove error circles if valid
pub fn manage_node_update_status(
    pc_resource: Res<PureCircuitResource>,
    mut event_reader: EventReader<NodeStatusUpdate>,
    mut query_status: Query<&mut GateStatusComponent>,
) {
    for NodeStatusUpdate(ind) in event_reader.read() {
        let Some(GraphStruct {
            node: NodeValue::GateNode { state_type, .. },
            additional_info: ent,
        }) = pc_resource.0.graph.node_weight(*ind).copied()
        else {
            error!("Missing node {:?}", ind);
            continue;
        };
        if let Ok(mut val) = query_status.get_mut(ent) {
            val.0 = state_type;
        } else {
            error!("Missing entity {:?}", ent);
        }
    }
}

/// Update value of node
pub fn manage_node_update(
    pc_resource: Res<PureCircuitResource>,
    mut event_reader: EventReader<NodeUpdate>,
    query_child: Query<&Children, With<ValueComponent>>,
    query: Query<(), Without<ErrorCircle>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for NodeUpdate(ind) in event_reader.read() {
        let GraphStruct {
            node,
            additional_info,
        } = pc_resource.0.graph[*ind];

        match node {
            NodeValue::ValueNode(_) => {
                commands
                    .entity(additional_info)
                    .despawn_related::<Children>();
            }
            _ => {
                if let Ok(children) = query_child.get(additional_info) {
                    for c in children {
                        if query.get(*c).is_ok() {
                            commands.entity(*c).despawn();
                        }
                    }
                }
            }
        };
        commands
            .entity(additional_info)
            .with_children(|builder| value_spawner(builder, node.to_new(), &asset_server));
    }
}

/// Reset solutions
pub fn manage_solution_reset(
    mut event_reader: EventReader<SolutionReset>,
    mut sol_indx: ResMut<SolutionIndex>,
    mut sol_val: ResMut<SolutionSet>,
) {
    for _ in event_reader.read() {
        sol_indx.0 = None;
        sol_val.0 = None;
    }
}

/// Reset solution index
pub fn index_reset(mut event_reader: EventReader<IndexReset>, mut sol_indx: ResMut<SolutionIndex>) {
    for _ in event_reader.read() {
        if let Some(v) = &mut sol_indx.0 {
            *v = 0;
        }
    }
}
