use crate::{
    algo_execution::plugin::{ErrorMessage, IsAlgoCurrentlyRunning},
    state_management::{
        events::{BacktrackEvent, NodeStatusUpdate, NodeUpdate},
        state_init::PureCircuitResource,
    },
};
use bevy::prelude::*;
use itertools::Itertools;
use pure_circuit_lib::{gates::Value, solution_finders::backtracking::BacktrackAlgorithm};

pub(super) struct BacktrackPlugin;

const MAX_LIMIT: usize = 20;

type SolSetType = Option<Vec<Vec<Option<Value>>>>;

#[derive(Default, Clone, Copy, PartialEq, Eq, Resource)]
pub(crate) struct SolutionIndex(pub Option<usize>);

#[derive(Debug, Resource, Default, PartialEq)]
pub struct SolutionSet(pub SolSetType);

impl Plugin for BacktrackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SolutionIndex>()
            .init_resource::<SolutionSet>()
            .add_systems(
                Update,
                execute_backtrack_handler.run_if(resource_equals(IsAlgoCurrentlyRunning(false))),
            )
            .add_systems(
                Update,
                modify_index.run_if(
                    resource_changed::<SolutionIndex>
                        .and(not(resource_equals(SolutionIndex(None))))
                        .and(not(resource_equals(SolutionIndex(Some(0)))))
                        .and(not(resource_equals(SolutionSet(None))))
                        .and(resource_equals(IsAlgoCurrentlyRunning(false))),
                ),
            );
    }
}

pub(super) fn execute_backtrack_handler(
    mut event_back: EventReader<BacktrackEvent>,
    mut sol_set: ResMut<SolutionSet>,
    mut sol_index: ResMut<SolutionIndex>,
    mut algo_handle: ResMut<IsAlgoCurrentlyRunning>,
    pc_resource: Res<PureCircuitResource>,
    mut err_message: ResMut<ErrorMessage>,
) {
    for _ in event_back.read() {
        if pc_resource.0.get_value_count() > MAX_LIMIT {
            continue;
        }
        algo_handle.0 = true;
        match BacktrackAlgorithm.calculate(&pc_resource.0) {
            Ok(v) => {
                err_message.reset();
                sol_set.0 = Some(v);
                sol_index.0 = None;
            }
            Err(e) => {
                err_message.set(
                    "Unable to run backtrack method. Check if there are any invalid arity gates",
                );
                error!("{}", e.to_string());
                sol_set.0 = None;
                sol_index.0 = None;
            }
        }

        algo_handle.0 = false;
    }
}

fn modify_index(
    sol_index: Res<SolutionIndex>,
    sol_set: Res<SolutionSet>,
    mut pc_resource: ResMut<PureCircuitResource>,
    mut event_writer_status: EventWriter<NodeUpdate>,
    mut event_writer: EventWriter<NodeStatusUpdate>,
) {
    let sol_index = sol_index
        .0
        .expect("Should be safe by the system conditions");
    let sol_set = sol_set
        .0
        .as_ref()
        .expect("Should be safe by the system conditions");
    let Some(sol) = sol_set.get(sol_index - 1) else {
        warn!("Solution index does not match solution set");
        return;
    };
    if let Err(e) = pc_resource.0.from_backtrack_sol(sol) {
        error!("{}", e.to_string());
        return;
    }

    event_writer_status.write_batch(
        pc_resource
            .0
            .graph
            .node_indices()
            .map(NodeUpdate)
            .collect_vec(),
    );
    event_writer.write_batch(
        pc_resource
            .0
            .graph
            .node_indices()
            .filter(|p| pc_resource.0.graph[*p].into_node().is_gate())
            .map(NodeStatusUpdate)
            .collect_vec(),
    );
}
