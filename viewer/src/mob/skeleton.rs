use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy_image::Image;

use crate::{
    animation::AnimationConfig,
    model::{AnimationData, ModelData},
};

pub struct Skeleton {
    pub variant: &'static str,
}

impl ModelData for Skeleton {
    fn entity_type() -> &'static str {
        "skeleton"
    }

    fn model_path() -> &'static str {
        "models/skeleton.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load(format!("images/entity/{}.png", self.variant)))
    }
}

impl AnimationData for Skeleton {
    fn configs() -> HashMap<String, AnimationConfig> {
        [(
            "riding".to_string(),
            AnimationConfig::Single {
                path: format!("{}#Animation0", Self::model_path()),
                speed: 0.0,
                repeat: false,
                paused: true,
            },
        )]
        .into_iter()
        .collect()
    }
}
