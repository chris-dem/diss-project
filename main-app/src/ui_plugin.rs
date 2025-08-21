use crate::algo_execution::back::{SolutionIndex, SolutionSet};
use crate::algo_execution::plugin::{ErrorMessage, IsAlgoCurrentlyRunning};
use crate::state_management::events::{BacktrackEvent, ButtonEvoEvent, ButtonHillEvent};
use crate::state_management::mouse_state::EdgeManagementState;
use crate::state_management::node_addition_state::{GateMode, ValueState};
use crate::{misc::cycle_enum_state, state_management::mouse_state::MouseState};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use egui::{Color32, Frame, Stroke};
use pure_circuit_lib::gates::{Gate, Value};

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
    err_message: Res<ErrorMessage>,
    algo_status: Res<IsAlgoCurrentlyRunning>,
    mut contexts: EguiContexts,
    mut solution_index: ResMut<SolutionIndex>,
    // Event Writers
    mut event_writer_evo: EventWriter<ButtonEvoEvent>,
    mut event_writer_hill: EventWriter<ButtonHillEvent>,
    mut event_writer_back: EventWriter<BacktrackEvent>,
) -> Result {
    let ctx = contexts.ctx_mut();
    egui::Window::new("Settings").show(ctx, |ui| {
        egui::Grid::new("preview").show(ui, |ui| {
            let col = gate_state.get_col();
            let text = gate_state.to_string();
            let [r, g, b, _a] = Srgba::from(col).to_u8_array();
            let col = Color32::from_rgb(r, g, b);
            ui.label("Toggle Node Mode (Press m to togle):");
            ui.colored_label(col, text);
            ui.end_row();
            ui.label("Toggle Mouse Mode (Press [ to togle):");
            ui.label(mouse_state.to_string());
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
            ui.separator();
            ui.end_row();
            ui.label("Solution Finders");
            ui.end_row();
            if ui.button("1. HillClimb").clicked() {
                event_writer_hill.write(ButtonHillEvent);
            }
            ui.end_row();
            if ui.button("2. Genetic Algorithms").clicked() {
                event_writer_evo.write(ButtonEvoEvent);
            }
            ui.end_row();

            // TODO: ui.add_text(); for errors
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
    Ok(())
}
