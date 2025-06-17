use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

mod camera_plugin;
mod constants;
mod drawing_plugin;
mod misc;
mod state_management;
mod ui_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(ShapePlugin)
        .add_plugins(ui_plugin::UiPlugin)
        .add_plugins(state_management::state_init::StateManagementPlugin)
        .add_plugins(drawing_plugin::DrawingPlugin)
        .add_plugins(camera_plugin::CameraPlugin)
        .run();
}
