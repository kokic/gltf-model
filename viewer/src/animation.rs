use std::collections::HashMap;

use bevy::ecs::{component::Component, resource::Resource};

#[derive(Clone, Debug)]
pub enum AnimationConfig {
    Single {
        path: String,
        speed: f32,
        repeat: bool,
        paused: bool,
    },
    Blend {
        animations: Vec<(String, f32)>,
        speed: f32,
        repeat: bool,
        paused: bool,
    },
}

#[derive(Component, Clone)]
pub struct ModelAnimation(pub String);

impl ModelAnimation {
    pub fn new(config_name: &str) -> Self {
        Self(config_name.to_string())
    }
}

#[derive(Resource)]
pub struct AnimationConfigs(pub HashMap<String, AnimationConfig>);
