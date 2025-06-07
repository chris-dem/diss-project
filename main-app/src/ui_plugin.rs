use crate::constants::*;
use crate::state_management::node_addition_state::GateMode;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
use egui::Color32;

pub struct UiPlugin;

#[derive(Component)]
struct MainPassCube;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_systems(
            EguiContextPass,
            cycle_add_state.run_if(input_just_pressed(KeyCode::KeyM)),
        )
        .add_systems(EguiContextPass, render_ui_window);
    }
}

fn cycle_add_state(
    current_state: Res<State<GateMode>>,
    mut next_state: ResMut<NextState<GateMode>>,
) {
    next_state.set(current_state.toggle());
}

fn render_ui_window(ui_state: Res<State<GateMode>>, mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut();
    let (col, text) = match **ui_state {
        GateMode::Gate => (GCOLOUR, GATETEXT),
        GateMode::Value => (VCOLOUR, VALTEXT),
    };
    let [r, g, b, _a] = Srgba::from(col).to_u8_array();
    let col = Color32::from_rgb(r, g, b);
    egui::Window::new("Settings").show(ctx, |ui| {
        egui::Grid::new("preview").show(ui, |ui| {
            ui.label("Toggle Mode (Press m to toggle):");
            ui.colored_label(col, text);
            ui.end_row();
        });
    });
    Ok(())
}
