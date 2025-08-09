use std::{collections::HashMap, time::Duration};

use bevy::{
    animation::{
        graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex},
        transition::AnimationTransitions,
        AnimationClip, AnimationPlayer,
    },
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        hierarchy::Children,
        observer::Trigger,
        query::Without,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    log::{debug, error, info},
    scene::SceneInstanceReady,
};

use crate::{
    animation::{AnimationConfig, AnimationConfigs, EntityAnimation},
    entity::EntityConfig,
};

#[derive(Resource)]
pub struct AnimationAssets {
    pub basic_graph: Handle<AnimationGraph>,
    pub basic_animations: HashMap<String, AnimationNodeIndex>,
    pub blend_graphs: HashMap<String, (Handle<AnimationGraph>, Vec<AnimationNodeIndex>)>,
    pending_animations: Vec<(String, Handle<AnimationClip>)>,
}

impl AnimationAssets {
    pub fn new() -> Self {
        Self {
            basic_graph: Handle::default(),
            basic_animations: HashMap::new(),
            blend_graphs: HashMap::new(),
            pending_animations: Vec::new(),
        }
    }

    pub fn register_animation(&mut self, name: String, clip: Handle<AnimationClip>) {
        self.pending_animations.push((name, clip));
    }

    pub fn finalize_basic_graph(&mut self, graphs: &mut Assets<AnimationGraph>) {
        if !self.pending_animations.is_empty() {
            let clips: Vec<_> = self
                .pending_animations
                .iter()
                .map(|(_, clip)| clip.clone())
                .collect();
            let (basic_graph, basic_node_indices) = AnimationGraph::from_clips(clips);

            for (i, (name, _)) in self.pending_animations.iter().enumerate() {
                self.basic_animations
                    .insert(name.clone(), basic_node_indices[i]);
            }

            self.basic_graph = graphs.add(basic_graph);
            self.pending_animations.clear();
        }
    }
}

pub fn setup_entity_animation(
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

pub fn update_animations(
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
                speed,
                repeat,
                paused,
                ..
            } => {
                let Some(&animation_node) = animation_assets
                    .basic_animations
                    .get(&entity_animation.config_name)
                else {
                    error!(
                        "Animation '{}' not found in basic_animations! Available: {:?}",
                        entity_animation.config_name,
                        animation_assets.basic_animations.keys().collect::<Vec<_>>()
                    );
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
                    "Started single animation '{}' for entity {:?} (speed: {}, repeat: {})",
                    entity_animation.config_name, entity, speed, repeat
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
