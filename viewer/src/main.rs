use std::collections::HashMap;

use bevy::prelude::*;
use viewer::{
    animation::{AnimationConfig, AnimationConfigs, ModelAnimation},
    components::texture_override::{self, TextureOverride},
    light,
    mob::{
        skeleton::Skeleton,
        villager::{self, Villager},
        zombie::Zombie,
    },
    model::{
        self,
        animation::AnimationAssets,
        assembly::{get_assembly_transform, Head, Localizable},
        AnimationData, ModelData,
    },
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
                model::animation::update_animations,
            ),
        )
        .add_observer(texture_override::observe)
        .add_observer(model::animation::setup_animation)
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

    // let apple = DroppingItem { item: "apple" };
    // commands.spawn((
    //     Transform::from_xyz(0.0, 1.0, 0.0),
    //     apple.default_bundle(&asset_server),
    // ));

    // let skeleton_variant: &[&'static str] =
    //     &["skeleton", "wither_skeleton", "stray"];

    // skeleton_variant
    //     .iter()
    //     .enumerate()
    //     .for_each(|(i, &variant)| {
    //         let skeleton = Skeleton { variant };
    //         let x = 4.0 + i as f32;
    //         commands
    //             .spawn((
    //                 Transform::from_xyz(x, 0.0, 0.0)
    //                     .looking_at(Vec3::new(x, 0.0, 1.0), Vec3::Y),
    //                 pig.default_bundle(&asset_server),
    //             ))
    //             .with_children(|parent| {
    //                 parent
    //                     .spawn((
    //                         Transform::from_xyz(0.0, 0.127, 0.0),
    //                         skeleton.default_bundle(&asset_server),
    //                         EntityAnimation("riding".to_string()),
    //                     ))
    //                     .with_children(|parent| {
    //                         parent.spawn((
    //                             witch_hat.default_bundle(&asset_server),
    //                             EntityAnimation("hide_nose".to_string()),
    //                         ));
    //                     });
    //             });
    //     });

    let oak = Oak;
    let pig = Pig;

    commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0)
            .looking_at(Vec3::new(0.0, 1.0, 1.0), Vec3::Y),
        oak.default_bundle(&asset_server),
        // ModelAnimation("idle".to_string()),
    ));

    commands.spawn((
        Transform {
            translation: Vec3::new(0.5, 2.0, 0.0),
            rotation: Quat::from_rotation_y(90.0_f32.to_radians())
                * Quat::from_rotation_x(90.0_f32.to_radians()),
            scale: Vec3::ONE,
        },
        pig.default_bundle(&asset_server),
    ));

    // let zombie = Zombie { variant: "zombie" };
    // let y = (32.0 + 1.0) / 16.0;
    // let z = (16.0 - 6.0) / 16.0;

    // let villager = Villager {
    //     variant: "villager",
    // };
    // commands.spawn((
    //     Transform {
    //         translation: Vec3::new(-3.2, 12.0 / 16.0, 0.0),
    //         rotation: Quat::from_rotation_z(180.0_f32.to_radians())
    //             * Quat::from_rotation_x(90.0_f32.to_radians())
    //             * Quat::from_rotation_y(90.0_f32.to_radians()),
    //         scale: Vec3::ONE,
    //     },
    //     villager.default_bundle(&asset_server),
    //     ModelAnimation(villager::ANIMATION_GENERAL.to_string()),
    // ));

    // commands.spawn((
    //     Transform::from_xyz(-2.0, y, z)
    //         .looking_at(Vec3::new(-2.0, y, 1.0), Vec3::Y)
    //         .with_scale(Vec3::splat(0.5)),
    //     zombie.default_bundle(&asset_server),
    //     ModelAnimation("baby_riding".to_string()),
    // ));

    // let mutant_zombie = MutantZombie;

    // commands.spawn((
    //     Transform::from_xyz(-2.0, 0.0, 0.0)
    //         .looking_at(Vec3::new(-2.0, 0.0, 1.0), Vec3::Y),
    //     mutant_zombie.default_bundle(&asset_server),
    // ));
    
}

type RegisterAnimationsFn = fn(
    &AssetServer,
    &mut Assets<AnimationGraph>,
    &mut AnimationAssets,
    &mut HashMap<String, AnimationConfig>,
);

const REGISTER_ANIMATIONS: &[RegisterAnimationsFn] = &[
    Villager::register,
    Pig::register,
    WitchHat::register,
    Skeleton::register,
    Squid::register,
    Zombie::register,
    Oak::register,
];

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

struct Oak;

impl ModelData for Oak {
    fn entity_type() -> &'static str {
        "oak"
    }

    fn model_path() -> &'static str {
        "models/oak.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load("images/entity/oak.png"))
    }
}

impl AnimationData for Oak {
    fn configs() -> HashMap<String, AnimationConfig> {
        [(
            "idle".to_string(),
            AnimationConfig::Single {
                path: format!("{}#Animation0", Self::model_path()),
                speed: 1.0,
                repeat: true,
                paused: false,
            },
        )]
        .into_iter()
        .collect()
    }
}

struct MutantZombie;

impl ModelData for MutantZombie {
    fn entity_type() -> &'static str {
        "mutant_zombie"
    }

    fn model_path() -> &'static str {
        "models/mutant_zombie.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load("images/entity/mutant_zombie.png"))
    }
}

struct Bed {
    pub variant: &'static str,
}

impl ModelData for Bed {
    fn entity_type() -> &'static str {
        "bed"
    }

    fn model_path() -> &'static str {
        "models/bed.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(
            asset_server
                .load(format!("images/entity/bed/{}.png", self.variant)),
        )
    }
}

struct Squid;

impl ModelData for Squid {
    fn entity_type() -> &'static str {
        "squid"
    }

    fn model_path() -> &'static str {
        "models/squid.gltf"
    }

    fn texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        Some(asset_server.load("images/entity/squid.png"))
    }
}

impl AnimationData for Squid {
    fn configs() -> HashMap<String, AnimationConfig> {
        [(
            "move".to_string(),
            AnimationConfig::Single {
                path: format!("{}#Animation0", Self::model_path()),
                speed: 1.0,
                repeat: true,
                paused: false,
            },
        )]
        .into_iter()
        .collect()
    }
}

struct Pig;

impl ModelData for Pig {
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

impl ModelData for WitchHat {
    fn entity_type() -> &'static str {
        "witch_hat"
    }

    fn model_path() -> &'static str {
        "models/witch_hat.gltf"
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
