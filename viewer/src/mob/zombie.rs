use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy_image::Image;

use crate::{
    animation::AnimationConfig,
    model::{AnimationData, BlendGraphConfig, ModelData},
};

pub struct Zombie {
    pub variant: &'static str,
}

impl ModelData for Zombie {
    fn entity_type() -> &'static str {
        "zombie"
    }

    fn model_path() -> &'static str {
        "models/zombie.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load(format!("images/entity/{}.png", self.variant)))
    }
}

impl AnimationData for Zombie {
    fn configs() -> HashMap<String, AnimationConfig> {
        [
            (
                "walk".to_string(),
                AnimationConfig::Single {
                    path: format!("{}#Animation0", Self::model_path()),
                    speed: 1.0,
                    repeat: true,
                    paused: false,
                },
            ),
            (
                "attack".to_string(),
                AnimationConfig::Single {
                    path: format!("{}#Animation1", Self::model_path()),
                    speed: 1.0,
                    repeat: true,
                    paused: false,
                },
            ),
            (
                "riding".to_string(),
                AnimationConfig::Single {
                    path: format!("{}#Animation2", Self::model_path()),
                    speed: 1.0,
                    repeat: false,
                    paused: true,
                },
            ),
            (
                "baby".to_string(),
                AnimationConfig::Single {
                    path: format!("{}#Animation3", Self::model_path()),
                    speed: 1.0,
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
            "baby_riding".to_string(),
            BlendGraphConfig {
                animations: vec![
                    ("baby".to_string(), 0.5),
                    ("riding".to_string(), 0.5),
                ],
                blend_factor: 0.5,
                speed: 0.0,
                repeat: true,
                paused: false,
            },
        )]
        .into_iter()
        .collect()
    }
}
