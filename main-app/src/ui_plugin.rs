use crate::{
    app_state::{AppState, GateMode, toggle_state},
    constants::*,
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{input::egui_wants_any_keyboard_input, EguiContextPass, EguiContexts, EguiPlugin};
use egui::Color32;

pub struct UiPlugin;

#[derive(Component)]
struct MainPassCube;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppState>()
            .add_systems(Startup, ui_setup)
            .add_plugins(EguiPlugin {
                enable_multipass_for_primary_context: true,
            })
            .add_systems(
                EguiContextPass,
                toggle_state.run_if(input_just_pressed(KeyCode::KeyM)),
            )
            .add_systems(EguiContextPass, render_ui_window);
    }
}

fn ui_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let default_material = ColorMaterial::from_color(VCOLOUR);
    let cube_handle = meshes.add(Rectangle::new(D_RADIUS, D_RADIUS));
    let main_material_handle = materials.add(default_material);

    // Main pass cube.
    commands
        .spawn((
            Mesh2d(cube_handle),
            MeshMaterial2d(main_material_handle),
            Transform::default(),
        ))
        .insert(MainPassCube);

    commands.spawn((Camera2d, Transform::default()));
}

fn render_ui_window(ui_state: Res<AppState>, mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut();
    let (col, text) = match ui_state.mode {
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
