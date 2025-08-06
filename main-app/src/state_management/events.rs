use bevy::prelude::*;
use petgraph::prelude::*;
use pure_circuit_lib::gates::{GraphStruct, NodeValue};

use crate::{
    drawing_plugin::GateStatusComponent, state_management::state_init::PureCircuitResource,
};

#[derive(Debug, Clone, Event)]
pub struct NodeStatusUpdate(pub NodeIndex);

#[derive(Debug, Clone, Event)]
pub struct EdgeAdditionEvent(pub EdgeIndex);

#[derive(Debug, Clone, Event)]
pub struct EdgeRemovalEvent(pub EdgeIndex);

pub struct EventManagerPlugin;

impl Plugin for EventManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EdgeRemovalEvent>()
            .add_event::<EdgeAdditionEvent>()
            .add_event::<NodeStatusUpdate>();
    }
}

pub fn manage_node_update_status(
    pc_resource: PureCircuitResource,
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
