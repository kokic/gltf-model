use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    animation::{AnimationConfig, AnimationConfigs, EntityAnimation},
    components::texture_override::TextureOverride,
};

/// 混合图配置
#[derive(Clone, Debug)]
pub struct BlendGraphConfig {
    pub name: String,
    pub animations: Vec<(String, f32)>, // (动画路径名称, 权重)
    pub blend_factor: f32,              // 混合系数
}

/// 实体配置 trait - 定义每个实体需要实现的接口
pub trait EntityConfig {
    /// 实体类型名称
    fn entity_type() -> &'static str;

    /// 模型文件路径
    fn model_path() -> &'static str;

    /// 纹理文件路径
    fn texture_path() -> &'static str;

    /// 动画路径配置 - 返回 (动画名称, 动画路径) 的数组
    fn animation_paths() -> &'static [(&'static str, &'static str)];

    /// 动画配置 - 返回具体的动画行为配置
    fn animation_configs() -> HashMap<String, AnimationConfig>;

    /// 混合图配置 - 返回需要创建的混合图
    fn blend_graph_configs() -> Vec<BlendGraphConfig> {
        Vec::new() // 默认为空，实体可以选择性实现
    }

    /// 默认动画配置名称
    fn default_animation() -> &'static str {
        "idle"
    }

    /// 注册实体的动画系统 - 在启动时调用
    fn register_animations(
        asset_server: &AssetServer,
        graphs: &mut Assets<AnimationGraph>,
        animation_assets: &mut AnimationAssets,
        animation_configs: &mut HashMap<String, AnimationConfig>,
    ) {
        // 注册基础动画
        let animation_paths = Self::animation_paths();
        for (name, path) in animation_paths {
            let clip = asset_server.load(*path);
            animation_assets.register_animation(name.to_string(), clip);
        }

        // 注册动画配置
        animation_configs.extend(Self::animation_configs());

        // 注册混合图
        for blend_config in Self::blend_graph_configs() {
            let blend_graph = Self::create_blend_graph(&blend_config, asset_server);
            let blend_handle = graphs.add(blend_graph.0);
            animation_assets
                .blend_graphs
                .insert(blend_config.name, (blend_handle, blend_graph.1));
        }
    }

    /// 创建混合图
    fn create_blend_graph(
        config: &BlendGraphConfig,
        asset_server: &AssetServer,
    ) -> (AnimationGraph, Vec<AnimationNodeIndex>) {
        let mut blend_graph = AnimationGraph::new();
        let blend_node = blend_graph.add_blend(config.blend_factor, blend_graph.root);

        let mut clip_indices = Vec::new();

        // 通过动画路径名称查找实际路径
        let animation_paths: HashMap<&str, &str> =
            Self::animation_paths().iter().copied().collect();

        for (anim_name, _weight) in &config.animations {
            if let Some(&path) = animation_paths.get(anim_name.as_str()) {
                let clip_node = blend_graph.add_clip(
                    asset_server.load(path),
                    1.0, // 权重在这里设为1.0，具体的权重由混合节点控制
                    blend_node,
                );
                clip_indices.push(clip_node);
            }
        }

        (blend_graph, clip_indices)
    }
}

/// 动画资源管理器
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

/// 实体生成器 - 用于在场景中创建实体
pub struct EntitySpawner;

impl EntitySpawner {
    /// 创建一个实体实例
    pub fn spawn<T: EntityConfig>(
        commands: &mut Commands,
        asset_server: &AssetServer,
        transform: Transform,
        animation_config: Option<&str>,
    ) -> Entity {
        let animation_name = animation_config.unwrap_or(T::default_animation());

        commands
            .spawn((
                transform,
                SceneRoot(asset_server.load(T::model_path())),
                TextureOverride(asset_server.load(T::texture_path())),
                EntityAnimation::new(animation_name),
            ))
            .id()
    }
}

/// 自动注册所有实体的动画系统
pub fn setup_all_entity_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // use crate::mob::{Pig, Skeleton, Villager, Zombie};

    let mut animation_assets = AnimationAssets::new();
    let mut animation_configs = HashMap::new();

    // 注册所有实体的动画
    crate::mob::villager::Villager::register_animations(
        &asset_server,
        &mut graphs,
        &mut animation_assets,
        &mut animation_configs,
    );
    // Skeleton::register_animations(
    //     &asset_server,
    //     &mut graphs,
    //     &mut animation_assets,
    //     &mut animation_configs,
    // );
    // Pig::register_animations(
    //     &asset_server,
    //     &mut graphs,
    //     &mut animation_assets,
    //     &mut animation_configs,
    // );
    // Zombie::register_animations(
    //     &asset_server,
    //     &mut graphs,
    //     &mut animation_assets,
    //     &mut animation_configs,
    // );

    // 完成基础图的创建
    animation_assets.finalize_basic_graph(&mut graphs);

    // 插入资源
    commands.insert_resource(animation_assets);
    commands.insert_resource(AnimationConfigs(animation_configs));
}
