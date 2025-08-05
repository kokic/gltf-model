use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    animation::AnimationConfig,
    entity::{BlendGraphConfig, EntityConfig},
};

pub struct Villager;

pub const ANIMATION_GENERAL: &str = "villager_general";
pub const ANIMATION_MOVE: &str = "villager_move";

impl EntityConfig for Villager {
    fn entity_type() -> &'static str {
        "villager"
    }

    fn model_path() -> &'static str {
        "models/villager.gltf#Scene0"
    }

    fn texture_path() -> &'static str {
        "images/entity/villager.png"
    }

    fn animation_paths() -> &'static [(&'static str, &'static str)] {
        &[
            (ANIMATION_GENERAL, "models/villager.gltf#Animation2"),
            (ANIMATION_MOVE, "models/villager.gltf#Animation3"),
        ]
    }

    fn animation_configs() -> HashMap<String, AnimationConfig> {
        let mut configs = HashMap::new();

        configs.insert(
            ANIMATION_GENERAL.to_string(),
            AnimationConfig::Single {
                animation_path: ANIMATION_GENERAL.to_string(),
                speed: 0.0,
                repeat: false,
                paused: true,
            },
        );

        configs.insert(
            ANIMATION_MOVE.to_string(),
            AnimationConfig::Single {
                animation_path: ANIMATION_MOVE.to_string(),
                speed: 2.0,
                repeat: true,
                paused: false,
            },
        );

        configs.insert(
            "villager_blend".to_string(),
            AnimationConfig::Blend {
                animations: vec![
                    (ANIMATION_GENERAL.to_string(), 0.5),
                    (ANIMATION_MOVE.to_string(), 0.5),
                ],
                speed: 1.0,
                repeat: true,
                paused: false,
            },
        );

        configs
    }

    fn blend_graph_configs() -> Vec<BlendGraphConfig> {
        vec![BlendGraphConfig {
            name: "villager_blend".to_string(),
            animations: vec![
                (ANIMATION_GENERAL.to_string(), 0.5),
                (ANIMATION_MOVE.to_string(), 0.5),
            ],
            blend_factor: 0.5,
        }]
    }

    fn default_animation() -> &'static str {
        ANIMATION_GENERAL
    }
}
