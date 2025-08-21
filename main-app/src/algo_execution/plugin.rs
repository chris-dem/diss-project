use bevy::prelude::*;
use itertools::Itertools;
use pure_circuit_lib::solution_finders::{
    self,
    evo_search::{EvoParamSet, HillParamSet, SolverEvo, SolverHillClimb},
    solver_trait::SolverTrait,
};

use crate::{
    algo_execution::back::BacktrackPlugin,
    state_management::{
        events::{ButtonEvoEvent, ButtonHillEvent, IndexReset, NodeStatusUpdate, NodeUpdate},
        state_init::PureCircuitResource,
    },
};

#[derive(Clone, Resource, Default, PartialEq, Eq)]
pub struct ErrorMessage(pub(crate) Option<String>);

impl ErrorMessage {
    pub(super) fn reset(&mut self) {
        self.0 = None;
    }

    pub(super) fn set(&mut self, other: &str) {
        self.0 = Some(other.into());
    }
}

pub struct AlgoPlugin;

impl Plugin for AlgoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BacktrackPlugin)
            .init_resource::<ErrorMessage>()
            .init_resource::<IsAlgoCurrentlyRunning>()
            .add_systems(
                Update,
                execute_evo_climbing.run_if(resource_equals(IsAlgoCurrentlyRunning(false))),
            )
            .add_systems(
                Update,
                execute_hill_climbing.run_if(resource_equals(IsAlgoCurrentlyRunning(false))),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Resource, Default)]
pub struct IsAlgoCurrentlyRunning(pub bool);

fn execute_hill_climbing(
    mut event_reader_hill: EventReader<ButtonHillEvent>,
    mut pc_resource: ResMut<PureCircuitResource>,
    mut err_message: ResMut<ErrorMessage>,
    mut algo_handle: ResMut<IsAlgoCurrentlyRunning>,
    mut event_writer_status: EventWriter<NodeUpdate>,
    mut event_writer: EventWriter<NodeStatusUpdate>,
    mut event_idx_writer: EventWriter<IndexReset>,
) {
    let solver = SolverHillClimb::default();
    for _ in event_reader_hill.read() {
        algo_handle.0 = true;
        let Some(func) = pc_resource.0.to_fitness_function() else {
            err_message.set(
                "Unable to create fitness function. Check if there are any invalid arity gates",
            );
            algo_handle.0 = false;
            return;
        };
        let count = pc_resource.0.count_values();
        let param_set = HillParamSet::build(
            solution_finders::evo_search::Instance::new(func, count),
            None,
        );
        match solver.find_solution(param_set) {
            Ok(e) => {
                if pc_resource.0.from_chromosone(&e.chromosone).is_none() {
                    error!("Failed to import chromosone");
                } else {
                    info!("PC has been successfully imported");
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
                err_message.reset();
            }
            Err(e) => error!("{}", e.to_string()),
        }

        event_idx_writer.write_default();
        algo_handle.0 = false;
    }
}

fn execute_evo_climbing(
    mut event_reader_hill: EventReader<ButtonEvoEvent>,
    mut pc_resource: ResMut<PureCircuitResource>,
    mut err_message: ResMut<ErrorMessage>,
    mut algo_handle: ResMut<IsAlgoCurrentlyRunning>,
    mut event_writer_status: EventWriter<NodeUpdate>,
    mut event_writer: EventWriter<NodeStatusUpdate>,
    mut event_idx_writer: EventWriter<IndexReset>,
) {
    let solver = SolverEvo::default();
    for _ in event_reader_hill.read() {
        algo_handle.0 = true;
        let Some(func) = pc_resource.0.to_fitness_function() else {
            err_message.set(
                "Unable to create fitness function. Check if there are any invalid arity gates",
            );
            algo_handle.0 = false;
            return;
        };
        let count = pc_resource.0.count_values();
        let param_set = EvoParamSet::build(
            solution_finders::evo_search::Instance::new(func, count),
            None,
        );
        match solver.find_solution(param_set) {
            Ok(e) => {
                if pc_resource.0.from_chromosone(&e.chromosone).is_none() {
                    error!("Failed to import chromosone");
                } else {
                    info!("PC has been successfully imported");
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
                    err_message.reset();
                }
            }
            Err(e) => error!("{}", e.to_string()),
        }

        event_idx_writer.write_default();
        algo_handle.0 = false;
    }
}
