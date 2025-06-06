use bevy::prelude::*;

mod constants;
mod ui_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ui_plugin::UiPlugin)
        .run();
}
