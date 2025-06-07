use bevy::prelude::*;

mod constants;
mod ui_plugin;
mod drawing_plugin;
mod app_state;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ui_plugin::UiPlugin)
        .add_plugins(drawing_plugin::DrawingPlugin)
        .run();
}
