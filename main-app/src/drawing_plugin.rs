use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    constants::{D_RADIUS, GCOLOUR, VCOLOUR},
    state_management::{mouse_state::MouseState, node_addition_state::GateMode},
};

pub struct DrawingPlugin;

#[derive(Component, Debug, Clone, Copy)]
pub struct MouseCircle;

#[derive(Component, Debug, Clone, Copy)]
pub struct MainCamera;

fn drawing_setup(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, drawing_setup)
            .add_systems(OnEnter(MouseState::Node), draw_setup)
            .add_systems(Update, hover_draw.run_if(in_state(MouseState::Node)))
            .add_systems(
                Update,
                click_draw
                    .run_if(in_state(MouseState::Node))
                    .run_if(input_just_pressed(MouseButton::Left)),
            )
            .add_systems(OnExit(MouseState::Node), draw_cleanup);
    }
}

fn hover_draw(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    gate_mode: Res<State<GateMode>>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;
    let Ok(window) = window.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // To test Camera::world_to_viewport, convert result back to viewport space and then back to world space.
    let Ok(viewport_check) = camera.world_to_viewport(camera_transform, world_pos.extend(0.0))
    else {
        return;
    };
    let Ok(world_check) = camera.viewport_to_world_2d(camera_transform, viewport_check.xy()) else {
        return;
    };
    let col = match **gate_mode {
        GateMode::Gate => GCOLOUR,
        GateMode::Value => VCOLOUR,
    };

    // Should be the same as world_pos
    gizmos.circle_2d(world_check, D_RADIUS, col);
}
fn draw_setup() {}
fn click_draw() {}
fn draw_cleanup() {}

#[cfg(test)]
mod tests {
    use super::*;

    mod startup_tests {
        use std::any::{Any, TypeId};

        use bevy::state::app::StatesPlugin;

        use crate::state_management::node_addition_state::GateMode;

        use super::*;

        fn setup() -> App {
            let mut app = App::new();
            app.add_plugins(StatesPlugin)
                .insert_state(MouseState::Node)
                .init_state::<GateMode>()
                .add_plugins(DrawingPlugin);

            app
        }

        #[test]
        fn should_spawn_a_mesh() {
            let mut app = setup();
            let query = app
                .world_mut()
                .try_query_filtered::<&Mesh2d, With<MouseCircle>>();
            assert!(query.is_some(), "Must contain elements");
            assert!(
                query.unwrap().query(app.world()).iter().len() == 1,
                "Should be only one"
            );
        }

        #[test]
        fn should_spawn_cicle_with_expected_properties() {
            let mut app = setup();
            let entity = app
                .world_mut()
                .query_filtered::<&Mesh2d, With<MouseCircle>>()
                .single(app.world());

            let mesh_manager = app
                .world()
                .get_resource::<Assets<Mesh>>()
                .expect("To exist");

            assert!(entity.is_ok(), "Error fetching single element");

            let mesh = entity.unwrap();
            assert!(
                mesh.type_id() == TypeId::of::<Circle>(),
                "Types should match"
            );
            assert!(
                mesh_manager.get(mesh.id()).is_some(),
                "Expect element to be declared"
            );
        }
    }
}
