use crate::algo_execution::back::{SolutionIndex, SolutionSet};
use crate::algo_execution::plugin::{ErrorMessage, EvoParam, HillParam, IsAlgoCurrentlyRunning};
use crate::state_management::events::{BacktrackEvent, ButtonEvoEvent, ButtonHillEvent};
use crate::state_management::mouse_state::EdgeManagementState;
use crate::state_management::node_addition_state::{GateMode, ValueState};
use crate::state_management::state_init::PureCircuitResource;
use crate::{misc::cycle_enum_state, state_management::mouse_state::MouseState};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use egui::{Color32, Frame, Stroke, Ui};
use genetic_algorithm::crossover::{self, CrossoverMultiPoint, CrossoverUniform, CrossoverWrapper};
use genetic_algorithm::mutate::{MutateMultiGene, MutateWrapper};
use genetic_algorithm::select::{SelectElite, SelectTournament, SelectWrapper};
use genetic_algorithm::strategy::prelude::HillClimbVariant;
use pure_circuit_lib::gates::{Gate, Value};
use pure_circuit_lib::solution_finders::evo_search::NewType;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        // .add_plugins(WorldInspectorPlugin::new())
        .add_systems(
            EguiContextPass,
            cycle_enum_state::<GateMode>.run_if(input_just_pressed(KeyCode::KeyG)),
        )
        .add_systems(
            EguiContextPass,
            cycle_enum_state::<ValueState<Value>>.run_if(input_just_pressed(KeyCode::KeyV)),
        )
        .add_systems(
            EguiContextPass,
            cycle_enum_state::<ValueState<Gate>>.run_if(input_just_pressed(KeyCode::KeyB)),
        )
        .add_systems(EguiContextPass, render_ui_window);
    }
}

#[derive(Component)]
struct MainPassCube;

#[allow(clippy::too_many_arguments)]
fn render_ui_window(
    gate_state: Res<State<GateMode>>,
    mouse_state: Res<State<MouseState>>,
    edge_management_state: Res<State<EdgeManagementState>>,
    value_mode: Res<State<ValueState<Value>>>,
    gate_mode: Res<State<ValueState<Gate>>>,
    solution_set: Res<SolutionSet>,
    pc_resource: Res<PureCircuitResource>,
    err_message: Res<ErrorMessage>,
    algo_status: Res<IsAlgoCurrentlyRunning>,
    mut hill_params: ResMut<HillParam>,
    mut evo_params: ResMut<EvoParam>,
    mut contexts: EguiContexts,
    mut solution_index: ResMut<SolutionIndex>,
    // Event Writers
    mut event_writer_evo: EventWriter<ButtonEvoEvent>,
    mut event_writer_hill: EventWriter<ButtonHillEvent>,
    mut event_writer_back: EventWriter<BacktrackEvent>,
) -> Result {
    let ctx = contexts.ctx_mut();
    let vals = pc_resource.0.count_values();
    egui::Window::new("Settings").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("preview").show(ui, |ui| {
                let col = gate_state.get_col();
                let text = gate_state.to_string();
                let [r, g, b, _a] = Srgba::from(col).to_u8_array();
                let col = Color32::from_rgb(r, g, b);
                ui.label("Toggle Mouse Mode (Press [ to togle):");
                ui.label(mouse_state.to_string());
                ui.end_row();
                ui.label("Toggle Node Mode (Press m to togle):");
                ui.colored_label(col, text);
                ui.end_row();
                ui.label("Toggle Value Mode (Press V to togle):");
                ui.label(value_mode.to_string());
                ui.end_row();
                ui.label("Toggle Gate Mode (Press B to togle):");
                ui.label(gate_mode.to_string());
                ui.end_row();
                ui.label("Toggle Edge Mode (Press J to togle):");
                ui.label(edge_management_state.to_string());
                ui.end_row();
                ui.label("To move camera, press L-CTRL and drag");
                ui.end_row();
                ui.label("To place a node: Mouse Mode = Node and Hold A");
                ui.end_row();
                ui.label("To place an edge: Mouse Mode = Edge and select two nodes");
                ui.end_row();
                ui.separator();
                ui.end_row();
                ui.heading("Solution Finders");
                ui.end_row();
                ui.separator();
                ui.end_row();
                hill_climb_ui(ui, &mut event_writer_hill, &mut hill_params);
                ui.separator();
                ui.end_row();
                genetic_algorithm_ui(ui, &mut event_writer_evo, &mut evo_params, vals);

                ui.separator();
                ui.end_row();
                if ui.button("Backtrack algorithm").clicked() {
                    event_writer_back.write(BacktrackEvent);
                }
                ui.end_row();
                if let Some(lim) = solution_set.0.as_ref().map(|x| x.len()) {
                    ui.horizontal(|ui| {
                        // Left button with frame
                        Frame::new()
                            .fill(Color32::from_rgb(40, 40, 40))
                            .inner_margin(egui::Margin {
                                left: 10,
                                right: 10,
                                top: 5,
                                bottom: 5,
                            })
                            .show(ui, |ui| {
                                if ui.button("<").clicked() {
                                    let v = solution_index.0.unwrap_or(0);
                                    solution_index.0 = Some(v.abs_diff(1));
                                }
                            });

                        // Center label with frame
                        let label = format!("{}/{}", solution_index.0.unwrap_or_default(), lim);
                        Frame::new()
                            .fill(Color32::from_rgb(50, 50, 50))
                            .stroke(Stroke::new(2.0, Color32::WHITE))
                            .inner_margin(egui::Margin {
                                left: 10,
                                right: 10,
                                top: 5,
                                bottom: 5,
                            })
                            .show(ui, |ui| {
                                ui.label(label);
                            });

                        // Right button with frame
                        Frame::new()
                            .fill(Color32::from_rgb(40, 40, 40))
                            .inner_margin(egui::Margin {
                                left: 10,
                                right: 10,
                                top: 5,
                                bottom: 5,
                            })
                            .show(ui, |ui| {
                                if ui.button(">").clicked() {
                                    let v = solution_index.0.unwrap_or(0);
                                    solution_index.0 = Some(lim.min(v + 1));
                                }
                            });
                    });
                    ui.end_row();
                }

                ui.separator();
                ui.end_row();
                ui.horizontal(|ui| {
                    ui.label("Algorithm status:");
                    let label = if algo_status.0 {
                        "Running"
                    } else {
                        "Not Running"
                    };
                    ui.label(label);
                });
                ui.end_row();
                ui.separator();
                ui.end_row();
                ui.vertical(|ui| {
                    ui.label("Error message");
                    if let Some(s) = err_message.0.as_ref() {
                        ui.label(s);
                    }
                })
            });
        });
    });
    Ok(())
}

fn hill_climb_ui(
    ui: &mut Ui,
    event_writer_hill: &mut EventWriter<ButtonHillEvent>,
    hill_params: &mut HillParam,
) {
    if ui.button("1. HillClimb").clicked() {
        event_writer_hill.write(ButtonHillEvent);
    }
    ui.end_row();
    ui.vertical(|ui| {
        ui.label("Number of runs:");
        ui.add(egui::DragValue::new(&mut hill_params.0.num_of_runs).range(1..=50usize));
        ui.end_row();
        ui.label("Population Size size:");
        ui.add(egui::DragValue::new(&mut hill_params.0.population_size).range(10..=500usize));
        ui.end_row();

        ui.vertical(|ui| {
            ui.label("Hill Climbing Strategy: ");
            ui.radio_value(
                &mut hill_params.0.hill_variant,
                NewType(HillClimbVariant::Stochastic),
                "Stochastic",
            );
            ui.radio_value(
                &mut hill_params.0.hill_variant,
                NewType(HillClimbVariant::SteepestAscent),
                "Steepest Ascent",
            )
        })
    });
    ui.end_row();
}

fn genetic_algorithm_ui(
    ui: &mut Ui,
    event_writer_evo: &mut EventWriter<ButtonEvoEvent>,
    evo_params: &mut EvoParam,
    num_of_values: usize,
) {
    if ui.button("2. Genetic Algorithms").clicked() {
        event_writer_evo.write(ButtonEvoEvent);
    }
    ui.end_row();
    ui.vertical(|ui| {
        ui.label("Number of runs:");
        ui.add(egui::DragValue::new(&mut evo_params.0.num_of_runs).range(1..=50usize));
        ui.end_row();
        ui.label("Population Size size:");
        ui.add(egui::DragValue::new(&mut evo_params.0.population_size).range(10..=500usize));
        ui.end_row();
        selection_ui(ui, evo_params);
        crossover_ui(ui, evo_params, num_of_values);
        mutation_ui(ui, evo_params, num_of_values);
    });
    ui.end_row();
}

fn selection_ui(ui: &mut Ui, evo_params: &mut EvoParam) {
    ui.label("Genetic algorithm Selection Strategy: ");
    ui.vertical(|ui| {
        // Elite
        ui.radio_value(
            &mut evo_params.0.selection,
            NewType(genetic_algorithm::select::SelectWrapper::Elite(
                SelectElite::new(0.05, 0.02),
            )),
            "Select Elite",
        );
        ui.end_row();

        if let SelectWrapper::Elite(SelectElite {
            replacement_rate,
            elitism_rate,
        }) = &mut evo_params.0.selection.0
        {
            egui::Frame::new()
                .stroke(Stroke::new(1., Color32::WHITE))
                .show(ui, |ui| {
                    ui.label("Replacement rate:");
                    ui.add(egui::DragValue::new(replacement_rate).range(0.01..=0.9f64));
                    ui.end_row();

                    ui.label("Elitism rate:");
                    ui.add(egui::DragValue::new(elitism_rate).range(0.01..=0.9f64));
                    ui.end_row();
                });
        }
        // Tournament
        ui.radio_value(
            &mut evo_params.0.selection,
            NewType(genetic_algorithm::select::SelectWrapper::Tournament(
                SelectTournament::new(0.05, 0.02, 5),
            )),
            "Select Tournament",
        );
        ui.end_row();
        if let SelectWrapper::Tournament(SelectTournament {
            replacement_rate,
            elitism_rate,
            tournament_size,
        }) = &mut evo_params.0.selection.0
        {
            egui::Frame::new()
                .stroke(Stroke::new(1., Color32::WHITE))
                .show(ui, |ui| {
                    ui.label("Replacement rate:");
                    ui.add(egui::DragValue::new(replacement_rate).range(0.01..=0.9f64));
                    ui.end_row();

                    ui.label("Elitism rate:");
                    ui.add(egui::DragValue::new(elitism_rate).range(0.01..=0.9f64));
                    ui.end_row();

                    ui.label("Tournament size:");
                    ui.add(egui::DragValue::new(tournament_size).range(10..=100));
                    ui.end_row();
                });
        }
    });
    ui.end_row();
}

fn crossover_ui(ui: &mut Ui, evo_params: &mut EvoParam, num_of_values: usize) {
    ui.label("Genetic algorithm Crossover Strategy: ");
    ui.vertical(|ui| {
        // Uniform
        ui.radio_value(
            &mut evo_params.0.crossover,
            NewType(CrossoverWrapper::Uniform(crossover::CrossoverUniform::new(
                0.5, 0.1,
            ))),
            "Uniform Selection",
        );
        ui.end_row();

        if let CrossoverWrapper::Uniform(CrossoverUniform {
            crossover_rate,
            selection_rate,
            ..
        }) = &mut evo_params.0.crossover.0
        {
            egui::Frame::new()
                .stroke(Stroke::new(1., Color32::WHITE))
                .show(ui, |ui| {
                    ui.label("Selection rate:(Percentage of the population to select)");
                    ui.add(egui::DragValue::new(selection_rate).range(0.01..=0.9f64));
                    ui.end_row();

                    ui.label("Crossover rate:(Probability of applying the crossover operator)");
                    ui.add(egui::DragValue::new(crossover_rate).range(0.01..=0.9f64));
                    ui.end_row();
                });
        }
        // Multipoint
        ui.radio_value(
            &mut evo_params.0.crossover,
            NewType(CrossoverWrapper::MultiPoint(
                crossover::CrossoverMultiPoint::new(0.5, 0.1, num_of_values / 5, true),
            )),
            "Crossover Multipoint",
        );
        ui.end_row();
        if let CrossoverWrapper::MultiPoint(CrossoverMultiPoint {
            crossover_rate,
            selection_rate,
            number_of_crossovers,
            allow_duplicates,
            ..
        }) = &mut evo_params.0.crossover.0
        {
            egui::Frame::new()
                .stroke(Stroke::new(1., Color32::WHITE))
                .show(ui, |ui| {
                    ui.label("Selection rate:(Percentage of the population to select)");
                    ui.add(egui::DragValue::new(selection_rate).range(0.01..=0.9f64));
                    ui.end_row();

                    ui.label("Crossover rate:(Probability of applying the crossover operator)");
                    ui.add(egui::DragValue::new(crossover_rate).range(0.01..=0.9f64));
                    ui.end_row();

                    ui.label(
                        "Number of crossovers:(Probability of applying the crossover operator)",
                    );
                    ui.add(
                        egui::DragValue::new(number_of_crossovers).range(1..=num_of_values / 10),
                    );
                    ui.end_row();

                    ui.checkbox(allow_duplicates, "Should allow duplicates:");
                    ui.end_row();
                });
        }
    });
    ui.end_row();
}

fn mutation_ui(ui: &mut Ui, evo_params: &mut EvoParam, num_of_values: usize) {
    ui.label("Genetic algorithm Mutation Strategy: ");
    ui.vertical(|ui| {
        if let MutateWrapper::MultiGene(MutateMultiGene {
            number_of_mutations,
            mutation_probability,
            ..
        }) = &mut evo_params.0.mutate.0
        {
            egui::Frame::new()
                .stroke(Stroke::new(1., Color32::WHITE))
                .show(ui, |ui| {
                    ui.label("Number of mutations:");
                    ui.add(egui::DragValue::new(number_of_mutations).range(1..=num_of_values));
                    ui.end_row();

                    ui.label("Mutation probability:(Probality of mutating offspring)");
                    ui.add(egui::DragValue::new(mutation_probability).range(0.01..=0.9f64));
                    ui.end_row();
                });
        }
    });
    ui.end_row();
}
