use std::time::Duration;

use bevy::{prelude::*, scene::SceneInstanceReady};
use viewer::{
    animation::{AnimationConfig, AnimationConfigs, EntityAnimation},
    components::texture_override::{self},
    entity::{setup_all_entity_animations, AnimationAssets, EntitySpawner},
    light,
    mob::villager::{self, Villager},
    simple_control,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(
            Startup,
            (
                light::setup_simple_light,
                setup_all_entity_animations,
                setup_scene, 
            )
                .chain(),
        ) // 确保按顺序执行
        .add_systems(
            Update,
            (
                simple_control::cursor_grab_system,
                simple_control::player_movement_system,
                simple_control::player_look_system,
                update_animations,
            ),
        )
        .add_observer(texture_override::observe)
        .add_observer(setup_entity_animation)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    simple_control::setup(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 1.0, 3.0),
        Some(Vec3::new(0.0, 1.0, -1.0)),
    );

    EntitySpawner::spawn::<Villager>(
        &mut commands,
        &asset_server,
        Transform::from_xyz(2.0, 0.0, 0.0).looking_at(Vec3::new(2.0, 0.0, 1.0), Vec3::Y),
        Some("villager_blend"),
    );

    EntitySpawner::spawn::<Villager>(
        &mut commands,
        &asset_server,
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0.0, 0.0, 1.0), Vec3::Y),
        Some(villager::ANIMATION_GENERAL),
    );

    // EntitySpawner::spawn::<Skeleton>(
    //     &mut commands,
    //     &asset_server,
    //     Transform::from_xyz(-2.0, 0.0, 0.0).looking_at(Vec3::new(-2.0, 0.0, 1.0), Vec3::Y),
    //     Some("skeleton_ride"),
    // );
}

fn setup_entity_animation(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    animation_assets: Res<AnimationAssets>,
    animation_configs: Res<AnimationConfigs>,
    mut players: Query<(Entity, &mut AnimationPlayer), Without<EntityAnimation>>,
    entity_animation: Query<&EntityAnimation>,
    children: Query<&Children>,
) {
    let Ok(entity_animation) = entity_animation.get(trigger.target()) else {
        return;
    };

    let Some(config) = animation_configs.0.get(&entity_animation.config_name) else {
        error!(
            "Animation config '{}' not found!",
            entity_animation.config_name
        );
        return;
    };

    for descendant in children.iter_descendants(trigger.target()) {
        if let Ok((entity, _player)) = players.get_mut(descendant) {
            commands.entity(entity).insert(entity_animation.clone());

            match config {
                AnimationConfig::Single { .. } => {
                    commands
                        .entity(entity)
                        .insert(AnimationGraphHandle(animation_assets.basic_graph.clone()));
                }
                AnimationConfig::Blend { .. } => {
                    if let Some((blend_graph_handle, _)) = animation_assets
                        .blend_graphs
                        .get(&entity_animation.config_name)
                    {
                        commands
                            .entity(entity)
                            .insert(AnimationGraphHandle(blend_graph_handle.clone()));
                    }
                }
            }

            debug!(
                "Setup animation '{}' for entity {:?}",
                entity_animation.config_name, entity
            );
        }
    }
}

fn update_animations(
    mut commands: Commands,
    animation_assets: Res<AnimationAssets>,
    animation_configs: Res<AnimationConfigs>,
    mut players: Query<
        (Entity, &mut AnimationPlayer, &EntityAnimation),
        Without<AnimationTransitions>,
    >,
) {
    for (entity, mut player, entity_animation) in &mut players {
        let Some(config) = animation_configs.0.get(&entity_animation.config_name) else {
            continue;
        };

        match config {
            AnimationConfig::Single {
                animation_path,
                speed,
                repeat,
                paused,
            } => {
                let Some(&animation_node) = animation_assets.basic_animations.get(animation_path)
                else {
                    error!("Animation path '{}' not found!", animation_path);
                    continue;
                };

                let mut transitions = AnimationTransitions::new();
                let animation = transitions.play(&mut player, animation_node, Duration::ZERO);

                animation.set_speed(*speed);

                if *repeat {
                    animation.repeat();
                }

                if *paused {
                    player.pause_all();
                } else {
                    player.resume_all();
                }

                commands.entity(entity).insert(transitions);

                info!(
                    "Started single animation '{}' (path: {}) for entity {:?} (speed: {}, repeat: {})",
                    entity_animation.config_name, animation_path, entity, speed, repeat
                );
            }
            AnimationConfig::Blend {
                speed,
                repeat,
                paused,
                ..
            } => {
                if let Some((_, clip_indices)) = animation_assets
                    .blend_graphs
                    .get(&entity_animation.config_name)
                {
                    for &clip_node_index in clip_indices {
                        let animation = player.play(clip_node_index);
                        animation.set_speed(*speed);

                        if *repeat {
                            animation.repeat();
                        }
                    }

                    if *paused {
                        player.pause_all();
                    } else {
                        player.resume_all();
                    }

                    commands.entity(entity).insert(AnimationTransitions::new());

                    info!(
                        "Started blend animation '{}' for entity {:?} (speed: {}, repeat: {})",
                        entity_animation.config_name, entity, speed, repeat
                    );
                }
            }
        }
    }
}
