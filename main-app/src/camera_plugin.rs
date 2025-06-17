use bevy::{
    input::{
        common_conditions::input_pressed,
        mouse::{MouseMotion, MouseWheel},
    },
    prelude::*,
};

use crate::state_management::mouse_state::MouseState;

pub struct CameraPlugin;

#[derive(Component, Debug, Clone, Copy)]
pub struct MainCamera;

fn drawing_setup(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera, Camera::default()));
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, drawing_setup)
            .add_systems(FixedUpdate, modify_zoom)
            .add_systems(
                Update,
                move_camera
                    .run_if(in_state(MouseState::Hover))
                    .run_if(input_pressed(MouseButton::Left)),
            );
    }
}

fn modify_zoom(
    mut main_cam: Single<&mut Projection, With<MainCamera>>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    let delta = mouse_wheel.read().map(|e| e.y).sum::<f32>() * 0.1;
    if delta != 0. {
        if let Projection::Orthographic(proj) = &mut **main_cam {
            proj.scale *= (1.0 - delta * 0.1).clamp(0.5, 2.0);
        }
    }
}

fn move_camera(
    mut main_cam: Single<&mut Transform, With<MainCamera>>,
    mut mouse_delta: EventReader<MouseMotion>,
) {
    for MouseMotion { delta } in mouse_delta.read() {
        main_cam.translation.x -= delta.x;
        main_cam.translation.y += delta.y;
    }
}
