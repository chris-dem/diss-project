use bevy::{
    color::palettes::css::{GREEN, RED, YELLOW},
    prelude::*,
};
use itertools::Itertools;
use pure_circuit_lib::solution_finders::{
    self,
    evo_search::{HillParamSet, SolverHillClimb, SolverStruct},
    solver_trait::SolverTrait,
};

use crate::state_management::{
    events::{NodeStatusUpdate, NodeUpdate},
    state_init::PureCircuitResource,
};

pub struct AlgoPlugin;

impl Plugin for AlgoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    let hill_climb_button = spawn_nested_text_bundle(
        &mut commands,
        text_font.clone(),
        Color::Srgba(RED),
        UiRect::all(Val::Px(10.)),
        "Hill Climibing",
        (Val::Px(10.), Val::Px(10.)),
    );
    commands
        .entity(hill_climb_button)
        .observe(execute_hill_climbing);
    // spawn_nested_text_bundle(
    //     &mut commands,
    //     text_font.clone(),
    //     Color::Srgba(RED),
    //     UiRect::all(Val::Px(10.)),
    //     "Test 2",
    //     (Val::Px(70.), Val::Px(10.)),
    // );
}

fn execute_hill_climbing(
    _trigger: Trigger<Pointer<Click>>,
    mut pc_resource: ResMut<PureCircuitResource>,
    mut event_writer_status: EventWriter<NodeUpdate>,
    mut event_writer: EventWriter<NodeStatusUpdate>,
) {
    let solver = SolverHillClimb::default();
    let Some(func) = pc_resource.0.to_fitness_function() else {
        dbg!("Not computable");
        return;
    };
    let count = pc_resource.0.count_values();
    let param_set = HillParamSet::build(
        solution_finders::evo_search::Instance::new(func, count),
        None,
    );
    match solver.find_solution(param_set) {
        Ok(e) => {
            if pc_resource.0.from_chromosone(&e.chromosone).is_none() {
                error!("Failed to import chromosone");
            } else {
                info!("PC has been successfully imported");
                event_writer_status.write_batch(
                    pc_resource
                        .0
                        .graph
                        .node_indices()
                        .map(NodeUpdate)
                        .collect_vec(),
                );
                event_writer.write_batch(
                    pc_resource
                        .0
                        .graph
                        .node_indices()
                        .filter(|p| pc_resource.0.graph[*p].into_node().is_gate())
                        .map(NodeStatusUpdate)
                        .collect_vec(),
                );
            }
        }
        Err(e) => error!("{}", e.to_string()),
    }
}
fn spawn_nested_text_bundle(
    builder: &mut Commands,
    font: Handle<Font>,
    background_color: Color,
    margin: UiRect,
    text: &str,
    position: (Val, Val),
) -> Entity {
    builder
        .spawn((
            Node {
                margin,
                position_type: PositionType::Absolute,
                top: position.0,
                right: position.1,
                padding: UiRect::axes(Val::Px(5.), Val::Px(5.)),
                width: Val::Px(100.),
                height: Val::Px(50.),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                ..default()
            },
            BackgroundColor(background_color),
            ZIndex(100),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(text),
                TextFont { font, ..default() },
                TextColor::BLACK,
            ));
        })
        .id()
}
