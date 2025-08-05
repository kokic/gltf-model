use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::winit::UpdateMode;

use viewer::block::BuiltBlockID;
use viewer::built_block_mesh::{get_texture, isotropic_mesh, top_bottom_mesh};
use viewer::chunk::{BlockTextures, Chunk, SetBlockEvent, WorldPos};
use viewer::raycast::RaycastDebugInfo;
use viewer::simple_control::PlayerCamera;
use viewer::{atlas_enum::AtlasEnum, block::BlockRegistry, chunk::World};
use viewer::{bindless_material::BindlessMaterial, chunk::RegenerateMesh};

use viewer::gpu_fsc::GpuFeatureSupportChecker;
use viewer::gui_atlas::{self};
use viewer::{crosshair, debug_screen, raycast, simple_control};

pub fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // present_mode: bevy::window::PresentMode::AutoVsync,
                    present_mode: bevy::window::PresentMode::Fifo,
                    ..Default::default()
                }),
                ..Default::default()
            }),
    )
    .insert_resource(bevy::winit::WinitSettings {
        // fps = 1000 / wait_time_ms
        focused_mode: UpdateMode::Reactive {
            wait: std::time::Duration::from_millis(16),
            react_to_device_events: false,
            react_to_user_events: true,
            react_to_window_events: true,
        },
        unfocused_mode: UpdateMode::Reactive {
            wait: std::time::Duration::from_millis(100),
            react_to_device_events: false,
            react_to_user_events: true,
            react_to_window_events: true,
        },
    })
    .add_plugins((
        GpuFeatureSupportChecker,
        WireframePlugin::default(),
        MaterialPlugin::<BindlessMaterial>::default(),
    ));

    app.add_systems(Startup, (setup, debug_screen::setup))
        .add_systems(
            Update,
            (
                input_handler,
                simple_control::player_movement_system,
                simple_control::player_look_system,
                simple_control::cursor_grab_system,
                viewer::chunk::handle_set_block_events.run_if(on_event::<SetBlockEvent>),
                viewer::wireframe::toggle_wireframe
                    .run_if(on_event::<viewer::wireframe::ToggleWireframe>),
                viewer::chunk::update_dirty_chunks,
                raycast::update_outline_box,
            ),
        )
        // .add_systems(PostUpdate, ())
        .add_systems(FixedUpdate, (debug_screen::update_coordinate_display,));
    app.insert_resource(Time::<Fixed>::from_hz(10.0));
    app.insert_resource(BreakCooldown::default());
    app.insert_resource(RaycastDebugInfo::default());

    app.add_event::<viewer::wireframe::ToggleWireframe>()
        .add_event::<RegenerateMesh>()
        .add_event::<SetBlockEvent>();

    app.insert_resource(AmbientLight {
        brightness: 200.0,
        color: Color::srgb(0.8, 0.9, 1.0),
        ..default()
    })
    .insert_resource(BlockRegistry {
        grass: top_bottom_mesh(
            get_texture("grass_carried.png"),
            get_texture("dirt.png"),
            get_texture("grass_side_carried.png"),
        ),
        brick: isotropic_mesh(get_texture("brick.png")),
        dirt: isotropic_mesh(get_texture("dirt.png")),
        planks_oak: isotropic_mesh(get_texture("planks_oak.png")),
        wool_colored_orange: isotropic_mesh(get_texture("wool_colored_orange.png")),
    });

    app.run();
}

#[derive(Resource)]
struct BreakCooldown {
    timer: Timer,
}

impl Default for BreakCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }
}

fn setup(
    breg: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<BindlessMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut set_block_events: EventWriter<SetBlockEvent>,
) {
    let breg = breg.into_inner();

    let (icons_texture_handle, icons_atlas_layout_handle) =
        gui_atlas::IconsAtlas::setup_atlas(&asset_server, &mut texture_atlases);

    crosshair::setup(
        &mut commands,
        icons_texture_handle,
        icons_atlas_layout_handle,
    );

    let block_textures: Vec<Handle<Image>> = viewer::built_block_mesh::BLOCK_TEXTURES
        .iter()
        .map(|&path| asset_server.load(format!("images/blocks/{}", path)))
        .collect();

    let mut world = World::new();

    set_block_events.write_batch([SetBlockEvent {
        world_pos: WorldPos { x: 0, y: 0, z: 0 },
        block: BuiltBlockID::WoolColoredOrange,
    }]);

    viewer::chunk::spawn_chunk(
        &mut world,
        &mut commands,
        &mut meshes,
        &mut materials,
        &breg,
        viewer::chunk::ChunkPos {
            x: -1,
            y: -1,
            z: -1,
        },
        &block_textures,
        Some(BuiltBlockID::Brick),
    );

    world.create_chunk_now(
        &mut commands,
        &mut meshes,
        &mut materials,
        &breg,
        viewer::chunk::ChunkPos { x: 0, y: -1, z: -1 },
        &block_textures,
    );

    world.create_chunk_now(
        &mut commands,
        &mut meshes,
        &mut materials,
        &breg,
        viewer::chunk::ChunkPos { x: -2, y: -1, z: -1 },
        &block_textures,
    );

    world.create_chunk_now(
        &mut commands,
        &mut meshes,
        &mut materials,
        &breg,
        viewer::chunk::ChunkPos { x: -1, y: -1, z: 0 },
        &block_textures,
    );

    world.create_chunk_now(
        &mut commands,
        &mut meshes,
        &mut materials,
        &breg,
        viewer::chunk::ChunkPos { x: -1, y: -1, z: -2 },
        &block_textures,
    );

    commands.insert_resource(BlockTextures(block_textures));
    commands.insert_resource(world);

    simple_control::setup(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 2.0, 0.0),
        Some(Vec3::new(2.0, 1.0, -4.0)),
    );

    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.95, 0.8),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 3.0,
            std::f32::consts::PI / 4.0,
            0.0,
        )),
        bevy::pbr::CascadeShadowConfigBuilder {
            first_cascade_far_bound: 25.0,
            maximum_distance: 200.0,
            ..default()
        }
        .build(),
    ));

    // commands.spawn((
    //     DirectionalLight {
    //         illuminance: 2000.0,
    //         shadows_enabled: false,
    //         color: Color::srgb(0.7, 0.8, 1.0),
    //         ..default()
    //     },
    //     Transform::from_rotation(Quat::from_euler(
    //         EulerRot::XYZ,
    //         std::f32::consts::PI / 6.0,
    //         -std::f32::consts::PI / 2.0,
    //         0.0,
    //     )),
    // ));
}

fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<PlayerCamera>>,
    mut set_block_events: EventWriter<SetBlockEvent>,
    // mut break_block_writer: EventWriter<BreakBlock>,
    mut break_cooldown: ResMut<BreakCooldown>,
    time: Res<Time>,
    windows: Query<&Window>,
    world: Res<World>,
    chunks: Query<&mut Chunk>,
) {
    break_cooldown.timer.tick(time.delta());

    if keyboard_input.just_pressed(KeyCode::KeyG) {
        set_block_events.write_batch([
            SetBlockEvent {
                world_pos: WorldPos { x: 1, y: 0, z: 0 },
                block: BuiltBlockID::Grass,
            },
            SetBlockEvent {
                world_pos: WorldPos { x: 0, y: 1, z: 0 },
                block: BuiltBlockID::Dirt,
            },
            SetBlockEvent {
                world_pos: WorldPos { x: 0, y: 0, z: 1 },
                block: BuiltBlockID::WoolColoredOrange,
            },
        ]);
    }

    if mouse_input.pressed(MouseButton::Left) && break_cooldown.timer.finished() {
        if let Ok((camera_transform, camera)) = camera_query.single() {
            if let Ok(window) = windows.single() {
                if let Some((ray_origin, ray_direction)) =
                    raycast::get_camera_ray(camera_transform, camera, window)
                {
                    if let Some(hit) = raycast::precise_minecraft_raycast(
                        ray_origin,
                        ray_direction,
                        &world,
                        &chunks,
                        20.0,
                    ) {
                        set_block_events.write(SetBlockEvent {
                            world_pos: hit.position,
                            block: BuiltBlockID::Air,
                        });

                        println!("破坏方块: {:?} (距离: {:.2})", hit.position, hit.distance);

                        break_cooldown.timer.reset();
                    }
                }
            }
        }
    }

    if mouse_input.just_pressed(MouseButton::Right) {
        if let Ok((camera_transform, camera)) = camera_query.single() {
            if let Ok(window) = windows.single() {
                if let Some((ray_origin, ray_direction)) =
                    raycast::get_camera_ray(camera_transform, camera, window)
                {
                    if let Some(hit) = raycast::precise_minecraft_raycast(
                        ray_origin,
                        ray_direction,
                        &world,
                        &chunks,
                        20.0,
                    ) {
                        if let Some(place_pos) =
                            raycast::get_adjacent_empty_position(&hit, &world, &chunks)
                        {
                            set_block_events.write(SetBlockEvent {
                                world_pos: place_pos,
                                block: BuiltBlockID::Brick,
                            });
                        }
                    }
                }
            }
        }
    }
}
