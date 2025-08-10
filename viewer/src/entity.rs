pub mod animation;
pub mod assembly;
pub mod humanoid;

use bevy::prelude::*;
use std::collections::HashMap;

use crate::{animation::AnimationConfig, entity::animation::AnimationAssets};

pub trait EntityData {
    fn entity_type() -> &'static str;
    fn model_path() -> &'static str;

    fn texture(&self, _: &AssetServer) -> Option<Handle<Image>> {
        None
    }

    fn scene(&self, asset_server: &AssetServer) -> Handle<Scene> {
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(Self::model_path()))
    }
}

#[derive(Clone, Debug)]
pub struct BlendGraphConfig {
    pub animations: Vec<(String, f32)>,
    pub blend_factor: f32,
    pub speed: f32,
    pub repeat: bool,
    pub paused: bool,
}

pub trait AnimationData {
    fn configs() -> HashMap<String, AnimationConfig>;

    fn blend_graph_configs() -> HashMap<String, BlendGraphConfig> {
        HashMap::new()
    }

    fn default_animation() -> &'static str {
        "idle"
    }

    fn register(
        asset_server: &AssetServer,
        graphs: &mut Assets<AnimationGraph>,
        animation_assets: &mut AnimationAssets,
        animation_configs: &mut HashMap<String, AnimationConfig>,
    ) {
        let configs = Self::configs();
        let blend_configs = Self::blend_graph_configs();

        for (name, config) in &configs {
            if let AnimationConfig::Single { path, .. } = config {
                let clip = asset_server.load(path.as_str());
                animation_assets.register_animation(name.clone(), clip);
                info!("Registered animation: {} -> {}", name, path);
            }
        }

        animation_configs.extend(configs);

        for (blend_name, blend_config) in blend_configs {
            info!("Creating blend graph: {}", blend_name);

            let blend_graph = Self::create_blend_graph(
                &blend_config,
                asset_server,
                animation_configs,
            );
            let blend_handle = graphs.add(blend_graph.0);
            animation_assets
                .blend_graphs
                .insert(blend_name.clone(), (blend_handle, blend_graph.1));

            animation_configs.insert(
                blend_name.clone(),
                AnimationConfig::Blend {
                    animations: blend_config.animations,
                    speed: blend_config.speed,
                    repeat: blend_config.repeat,
                    paused: blend_config.paused,
                },
            );
        }
    }

    fn create_blend_graph(
        config: &BlendGraphConfig,
        asset_server: &AssetServer,
        animation_configs: &HashMap<String, AnimationConfig>,
    ) -> (AnimationGraph, Vec<AnimationNodeIndex>) {
        let mut blend_graph = AnimationGraph::new();
        let blend_node =
            blend_graph.add_blend(config.blend_factor, blend_graph.root);

        let mut clip_indices = Vec::new();

        for (anim_name, _weight) in &config.animations {
            if let Some(AnimationConfig::Single {
                path: animation_id, ..
            }) = animation_configs.get(anim_name)
            {
                info!(
                    "Adding clip to blend graph: {} -> {}",
                    anim_name, animation_id
                );
                let clip_node = blend_graph.add_clip(
                    asset_server.load(animation_id.as_str()),
                    1.0,
                    blend_node,
                );
                clip_indices.push(clip_node);
            } else {
                error!(
                    "Animation config not found for blend: {} (available: {:?})",
                    anim_name,
                    animation_configs.keys().collect::<Vec<_>>()
                );
            }
        }

        (blend_graph, clip_indices)
    }
}
