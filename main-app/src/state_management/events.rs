use bevy::prelude::*;
use petgraph::prelude::*;
use pure_circuit_lib::gates::{GraphStruct, NewNode, NodeValue};

use crate::{
    drawing_plugin::{GateStatusComponent, value_spawner},
    state_management::state_init::PureCircuitResource,
};

#[derive(Debug, Clone, Event)]
pub struct NodeStatusUpdate(pub NodeIndex);

#[derive(Debug, Clone, Event)]
pub struct NodeUpdate(pub NodeIndex);

pub struct EventManagerPlugin;

impl Plugin for EventManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, manage_node_update_status)
            .add_systems(Update, manage_node_update)
            .add_event::<NodeUpdate>()
            .add_event::<NodeStatusUpdate>();
    }
}

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

pub fn manage_node_update(
    pc_resource: Res<PureCircuitResource>,
    mut event_reader: EventReader<NodeUpdate>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for NodeUpdate(ind) in event_reader.read() {
        let GraphStruct {
            node,
            additional_info,
        } = pc_resource.0.graph[*ind];
        let mut ent = commands.entity(additional_info);

        ent.despawn_related::<Children>();
        ent.with_children(|builder| value_spawner(builder, node.to_new(), &asset_server));
    }
}
