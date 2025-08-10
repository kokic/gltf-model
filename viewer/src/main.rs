use std::collections::HashMap;

use bevy::prelude::*;
use viewer::{
    animation::{AnimationConfig, AnimationConfigs, EntityAnimation},
    components::texture_override::{self, TextureOverride},
    entity::{
        self,
        animation::AnimationAssets,
        assembly::{get_assembly_transform, Head, Localizable},
        AnimationData, EntityData,
    },
    light,
    mob::villager::{self, Villager},
    simple_control,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, light::setup_simple_light)
        .add_systems(
            Startup,
            (setup_all_entity_animations, setup_scene).chain(),
        )
        .add_systems(
            Update,
            (
                simple_control::cursor_grab_system,
                simple_control::player_movement_system,
                simple_control::player_look_system,
                entity::animation::update_animations,
            ),
        )
        .add_observer(texture_override::observe)
        .add_observer(entity::animation::setup_entity_animation)
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

    let pig = Pig;
    let villager = Villager;
    let witch_hat = WitchHat;

    commands
        .spawn((
            Transform::from_xyz(2.0, 0.0, 0.0)
                .looking_at(Vec3::new(2.0, 0.0, 1.0), Vec3::Y),
            TextureOverride(pig.texture(&asset_server)),
            SceneRoot(pig.scene(&asset_server)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Transform::from_xyz(0.0, 0.127, 0.25),
                    TextureOverride(villager.texture(&asset_server)),
                    SceneRoot(villager.scene(&asset_server)),
                    EntityAnimation(villager::ANIMATION_RIDING.to_string()),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Transform::default(),
                        TextureOverride(witch_hat.texture(&asset_server)),
                        SceneRoot(witch_hat.scene(&asset_server)),
                    ));
                });
            parent.spawn((
                get_assembly_transform::<Pig, Head, WitchHat, Head>(),
                TextureOverride(witch_hat.texture(&asset_server)),
                SceneRoot(witch_hat.scene(&asset_server)),
            ));
        });

    let apple = DroppingItem { item: "apple" };

    commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        TextureOverride(apple.texture(&asset_server)),
        SceneRoot(apple.scene(&asset_server)),
    ));
}

type RegisterAnimationsFn = fn(
    &AssetServer,
    &mut Assets<AnimationGraph>,
    &mut AnimationAssets,
    &mut HashMap<String, AnimationConfig>,
);

const REGISTER_ANIMATIONS: &[RegisterAnimationsFn] =
    &[Villager::register, Pig::register, WitchHat::register];

pub fn setup_all_entity_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut animation_assets = AnimationAssets::new();
    let mut animation_configs = HashMap::new();

    for register_fn in REGISTER_ANIMATIONS {
        register_fn(
            &asset_server,
            &mut graphs,
            &mut animation_assets,
            &mut animation_configs,
        );
    }

    animation_assets.finalize_basic_graph(&mut graphs);

    commands.insert_resource(animation_assets);
    commands.insert_resource(AnimationConfigs(animation_configs));
}

type ItemID = &'static str;

struct DroppingItem {
    item: ItemID,
}

impl EntityData for DroppingItem {
    fn entity_type() -> &'static str {
        "dropping_item"
    }

    fn model_path() -> &'static str {
        "models/dropping.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load(format!("images/items/{}.png", self.item)))
    }
}

struct Pig;

impl EntityData for Pig {
    fn entity_type() -> &'static str {
        "pig"
    }

    fn model_path() -> &'static str {
        "models/pig.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load("images/entity/pig.png"))
    }
}

impl AnimationData for Pig {
    fn configs() -> HashMap<String, AnimationConfig> {
        HashMap::new()
    }
}

impl Localizable<Head> for Pig {
    fn position() -> Vec3 {
        Vec3::new(-4.0, 8.0, -14.0)
    }
}

struct WitchHat;

impl EntityData for WitchHat {
    fn entity_type() -> &'static str {
        "witch_hat"
    }

    fn model_path() -> &'static str {
        "models/with_textures/witch_hat.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load("images/entity/witch.png"))
    }
}

impl AnimationData for WitchHat {
    fn configs() -> HashMap<String, AnimationConfig> {
        [(
            "hide_nose".to_string(),
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

impl Localizable<Head> for WitchHat {
    fn position() -> Vec3 {
        Vec3::new(-4.0, 24.0, -4.0)
    }
}
