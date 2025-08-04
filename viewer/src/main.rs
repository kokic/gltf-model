use viewer::simple_control;

use bevy::prelude::*;

// use crate::simple_controller::{Player, PlayerCamera};

const MODEL_PATH: &str = "models/pig.gltf";

fn main() {
    App::new()
        // .insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4096 })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.0,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, setup_scene_once_loaded)
        .add_systems(Update, keyboard_control)
        .add_systems(
            Update,
            (
                simple_control::cursor_grab_system,
                simple_control::player_movement_system,
                simple_control::player_look_system,
            ),
        )
        .run();
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(MODEL_PATH)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(MODEL_PATH)),
    ]);

    // Keep our animation graph in a Resource so that it can be inserted onto
    // the correct entity once the scene actually loads.
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph_handle,
    });

    simple_control::setup(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Fixed Viewport Camera
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-2.0, 2.0, -2.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
    // ));

    // Light
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::PI / 4.,
        )),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        bevy::pbr::CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));

    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL_PATH)),
    ));
}

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(
                &mut player,
                animations.animations[0],
                std::time::Duration::ZERO,
            )
            .repeat()
            .set_speed(2.0);

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
    }
}

fn keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for (mut player, mut transitions) in &mut animation_players {
        // let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
        //     continue;
        // };

        if keyboard_input.just_pressed(KeyCode::Enter) {
            *current_animation = (*current_animation + 1) % animations.animations.len();

            transitions
                .play(
                    &mut player,
                    animations.animations[*current_animation],
                    std::time::Duration::from_millis(250),
                )
                .repeat()
                .set_speed(2.0);
        }
    }
}
