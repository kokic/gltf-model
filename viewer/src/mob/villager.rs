use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    animation::AnimationConfig,
    entity::{
        assembly::{Head, Localizable}, humanoid::Humanoid, BlendGraphConfig,
        EntityConfig,
    },
};

pub struct Villager;

pub const ANIMATION_GENERAL: &str = "villager.general";
pub const ANIMATION_MOVE: &str = "villager.move";
pub const ANIMATION_GENERAL_MOVE: &str = "villager.general_move";
pub const ANIMATION_RIDING: &str = "villager.riding";

impl Localizable<Head> for Villager {
    fn position() -> Vec3 {
        <Humanoid as Localizable<Head>>::position()
    }
}

impl EntityConfig for Villager {
    fn entity_type() -> &'static str {
        "villager"
    }

    fn model_path() -> &'static str {
        "models/villager.gltf"
    }

    fn texture(asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load("images/entity/villager.png"))
    }

    fn animation_configs() -> HashMap<String, AnimationConfig> {
        [
            (
                ANIMATION_GENERAL.to_string(),
                AnimationConfig::Single {
                    path: format!("{}#Animation2", Self::model_path()),
                    speed: 0.0,
                    repeat: false,
                    paused: true,
                },
            ),
            (
                ANIMATION_MOVE.to_string(),
                AnimationConfig::Single {
                    path: format!("{}#Animation3", Self::model_path()),
                    speed: 2.0,
                    repeat: true,
                    paused: false,
                },
            ),
            (
                ANIMATION_RIDING.to_string(),
                AnimationConfig::Single {
                    path: format!("{}#Animation4", Self::model_path()),
                    speed: 0.0,
                    repeat: false,
                    paused: true,
                },
            ),
        ]
        .into_iter()
        .collect()
    }

    fn blend_graph_configs() -> HashMap<String, BlendGraphConfig> {
        [(
            ANIMATION_GENERAL_MOVE.to_string(),
            BlendGraphConfig {
                animations: vec![
                    (ANIMATION_GENERAL.to_string(), 0.5),
                    (ANIMATION_MOVE.to_string(), 0.5),
                ],
                blend_factor: 0.5,
                speed: 2.0,
                repeat: true,
                paused: false,
            },
        )]
        .into_iter()
        .collect()
    }

    fn default_animation() -> &'static str {
        ANIMATION_GENERAL
    }
}
