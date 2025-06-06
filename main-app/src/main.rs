use bevy::{
    ecs::observer::TriggerTargets, input::common_conditions::input_just_pressed, prelude::*,
};

// const D_RADIUS: f32 = 50.0f32;
const VCOLOUR: Color = Color::srgb_u8(47, 158, 68);
const GCOLOUR: Color = Color::srgb_u8(103, 65, 217);
const MARGIN: Val = Val::Px(12.);
static VALTEST: &str = "Value Mode";
static GATETEST: &str = "Gate Mode";

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

// Add necessary shapes
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.insert_resource(GateMode(CurrentChoice::Value));
    let bund = spawn_nested_text_bundle(&mut commands, VCOLOUR, UiRect::all(Val::Px(5.)), VALTEST);
    commands.entity(bund).insert(ColourPicker);
    commands
        .spawn(Node {
            // fill the entire window
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            padding: UiRect::all(MARGIN),
            row_gap: MARGIN,
            ..Default::default()
        })
        .add_child(bund);
}

fn toggle_status(mut query: ResMut<GateMode>) {
    *query = GateMode(query.0.opposite());
}

fn draw_col(
    mut mat: Single<(&mut BackgroundColor, &mut Children), With<ColourPicker>>,
    mut query_text: Query<&mut Text>,
    state: Res<GateMode>,
) {
    let (ref mut bg, ref mut children) = *mat;
    let text_children = query_text.get_mut(
        children
            .entities()
            .next()
            .expect("Should be loaded with text"),
    );
    if let Ok(mut value) = text_children {
        match &state.0 {
            CurrentChoice::Gate => {
                bg.0 = GCOLOUR;
                value.0 = GATETEST.into();
            }

            CurrentChoice::Value => {
                bg.0 = VCOLOUR;
                value.0 = VALTEST.into();
            }
        }
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

fn spawn_nested_text_bundle(
    builder: &mut Commands,
    background_color: Color,
    margin: UiRect,
    text: &str,
) -> Entity {
    builder
        .spawn((
            Node {
                margin,
                padding: UiRect::axes(Val::Px(5.), Val::Px(1.)),
                ..default()
            },
            BackgroundColor(background_color),
        ))
        .with_children(|builder| {
            builder.spawn((Text::new(text), TextFont::default(), TextColor::BLACK));
        })
        .id()
}
