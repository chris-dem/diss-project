use crate::constants::*;
use bevy::{
    ecs::observer::TriggerTargets, input::common_conditions::input_just_pressed, prelude::*,
};

pub struct UiPlugin;

#[derive(Resource, Default, Debug)]
pub struct UiState {
    mode: GateMode,
}

#[derive(Debug, Clone, Copy, Default)]
enum GateMode {
    #[default]
    Value,
    Gate,
}

impl GateMode {
    fn toggle(&self) -> Self {
        match self {
            Self::Value => Self::Gate,
            Self::Gate => Self::Value,
        }
    }
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiState::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                toggle_status.run_if(input_just_pressed(MouseButton::Left)),
            )
            .add_systems(Update, draw_col);
    }
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

#[derive(Component)]
struct ColourPicker;

// Add necessary shapes
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

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

fn toggle_status(mut query: ResMut<UiState>) {
    query.mode = query.mode.toggle();
}

fn draw_col(
    mut mat: Single<(&mut BackgroundColor, &mut Children), With<ColourPicker>>,
    mut query_text: Query<&mut Text>,
    state: Res<UiState>,
) {
    let (ref mut bg, ref mut children) = *mat;
    let text_children = query_text.get_mut(
        children
            .entities()
            .next()
            .expect("Should be loaded with text"),
    );
    if let Ok(mut value) = text_children {
        match &state.mode {
            GateMode::Gate => {
                bg.0 = GCOLOUR;
                value.0 = GATETEST.into();
            }

            GateMode::Value => {
                bg.0 = VCOLOUR;
                value.0 = VALTEST.into();
            }
        }
    }
}
