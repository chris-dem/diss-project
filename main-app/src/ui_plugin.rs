use crate::state_management::node_addition_state::{GateMode, ValueState};
use crate::{misc::cycle_enum_state, state_management::mouse_state::MouseState};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use egui::Color32;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(
            EguiContextPass,
            cycle_enum_state::<GateMode>.run_if(input_just_pressed(KeyCode::KeyG)),
        )
        .add_systems(
            EguiContextPass,
            cycle_enum_state::<ValueState>.run_if(input_just_pressed(KeyCode::KeyV)),
        )
        .add_systems(EguiContextPass, render_ui_window);
    }
}

#[derive(Component)]
struct MainPassCube;

fn render_ui_window(
    gate_state: Res<State<GateMode>>,
    mouse_state: Res<State<MouseState>>,
    value_mode: Res<State<ValueState>>,
    mut contexts: EguiContexts,
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
        });
    });
    Ok(())
}
