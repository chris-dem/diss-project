use bevy::{input::common_conditions::input_just_pressed, prelude::*};

const D_RADIUS: f32 = 50.0f32;
const BBDIMS: (f32, f32) = (50.0f32, 50.0f32);
const VCOLOUR: Color = Color::srgb_u8(47, 158, 68);
const GCOLOUR: Color = Color::srgb_u8(103, 65, 217);

#[derive(Debug, Clone, Copy)]
enum CurrentChoice {
    Value,
    Gate,
}

impl CurrentChoice {
    fn opposite(&self) -> Self {
        match self {
            CurrentChoice::Value => CurrentChoice::Gate,
            CurrentChoice::Gate => CurrentChoice::Value,
        }
    }
}

#[derive(Resource)]
struct GateMode(CurrentChoice);

#[derive(Component)]
struct ColourPicker;

#[derive(Resource)]
struct ValueResource(pub Handle<ColorMaterial>);

#[derive(Resource)]
struct GateResource(pub Handle<ColorMaterial>);

// Add necessary shapes
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let rect = meshes.add(Rectangle::new(BBDIMS.0, BBDIMS.1));
    commands.spawn(Camera2d);
    let hand_v = materials.add(VCOLOUR);
    materials.add(GCOLOUR);
    commands.spawn((
        Mesh2d(rect),
        MeshMaterial2d(hand_v),
        Transform::default(),
        ColourPicker,
    ));

    commands.insert_resource(GateMode(CurrentChoice::Value));
}

fn toggle_status(mut query: ResMut<GateMode>) {
    *query = GateMode(query.0.opposite());
}

fn draw_col(
    mat: Single<&MeshMaterial2d<ColorMaterial>, With<ColourPicker>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<GateMode>,
) {
    let m = materials
        .get_mut(&mat.0)
        .expect("No clue wtf this is doing");
    match &state.0 {
        CurrentChoice::Gate => *m = GCOLOUR.into(),
        CurrentChoice::Value => *m = VCOLOUR.into(),
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            toggle_status.run_if(input_just_pressed(MouseButton::Left)),
        )
        .add_systems(Update, draw_col)
        .run();
}
