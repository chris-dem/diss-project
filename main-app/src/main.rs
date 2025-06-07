use bevy::prelude::*;

mod state_management;
mod constants;
mod ui_plugin;
mod drawing_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ui_plugin::UiPlugin)
        .add_plugins(state_management::state_init::StateManagementPlugin)
        .add_plugins(drawing_plugin::DrawingPlugin)
        .run();
}
