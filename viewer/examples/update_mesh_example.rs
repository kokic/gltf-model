use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::winit::UpdateMode;
use bevy_meshem::{
    prelude::{mesh_grid, update_mesh, MeshMD, MeshingAlgorithm, VoxelChange},
    util::get_neighbor,
    Dimensions, VoxelMesh, VoxelRegistry,
};
use rand::prelude::*;

use strum::EnumCount;
use viewer::atlas_enum::AtlasEnum;
use viewer::bindless_material::{BindlessMaterial, MaterialUniforms};
use viewer::block::BuiltBlockID;
use viewer::built_block_mesh::{get_texture, isotropic_mesh, top_bottom_mesh, BLOCK_TEXTURES};

use viewer::gpu_fsc::GpuFeatureSupportChecker;
use viewer::gui_atlas::{self};
use viewer::{crosshair, position_display, simple_control};
// use bevy::window::CursorGrabMode;

const FACTOR: usize = 8;
const CHUNK_LEN: usize = FACTOR * FACTOR * FACTOR;

pub fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
    )
    .insert_resource(bevy::winit::WinitSettings {
        focused_mode: UpdateMode::Continuous,
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

    app.insert_resource(BlockRegistry {
        grass: top_bottom_mesh(
            get_texture("grass_carried.png"),
            get_texture("dirt.png"),
            get_texture("grass_side_carried.png"),
        ),
        dirt: isotropic_mesh(get_texture("dirt.png")),
        planks_oak: isotropic_mesh(get_texture("planks_oak.png")),
        wool_colored_orange: isotropic_mesh(get_texture("wool_colored_orange.png")),
    })
    .insert_resource(AmbientLight {
        brightness: 600.0,
        color: Color::WHITE,
        ..default()
    });

    app.add_systems(Startup, (setup, position_display::setup))
        .add_systems(
            Update,
            (
                viewer::wireframe::toggle_wireframe,
                input_handler,
                mesh_update,
                raycast_break_block,
            ),
        )
        .add_systems(
            Update,
            (
                simple_control::cursor_grab_system,
                simple_control::player_movement_system,
                simple_control::player_look_system,
            ),
        )
        .add_systems(Update, position_display::update_coordinate_display);

    app.add_event::<viewer::wireframe::ToggleWireframe>()
        .add_event::<RegenerateMesh>()
        .add_event::<BreakBlock>();

    app.run();
}

#[derive(Component)]
struct Meshy {
    meta: MeshMD<BuiltBlockID>,
    grid: [BuiltBlockID; CHUNK_LEN],
}

#[derive(Event, Default)]
struct RegenerateMesh;

#[derive(Event)]
struct BreakBlock {
    ray_origin: Vec3,
    ray_direction: Vec3,
}

/// Setting up everything to showcase the mesh.
fn setup(
    breg: Res<BlockRegistry>,
    mut commands: Commands,
    mut materials: ResMut<Assets<BindlessMaterial>>,
    // wireframe_config: ResMut<WireframeConfig>,
    // mut images: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let (icons_texture_handle, icons_atlas_layout_handle) =
        gui_atlas::IconsAtlas::setup_atlas(&asset_server, &mut texture_atlases);

    crosshair::setup(
        &mut commands,
        icons_texture_handle,
        icons_atlas_layout_handle,
    );

    // chunk
    let mut grid: Vec<BuiltBlockID> = vec![BuiltBlockID::Air; CHUNK_LEN];
    grid = grid
        .iter_mut()
        .enumerate()
        .map(|(i, _)| {
            let chunk_x = i % FACTOR;
            let chunk_y = (i / FACTOR) % FACTOR;
            let chunk_z = i / (FACTOR * FACTOR);

            if chunk_z >= FACTOR - 1 {
                BuiltBlockID::Grass
            } else if chunk_z >= FACTOR - 3 {
                BuiltBlockID::Dirt
            } else {
                let index = (chunk_x + chunk_y) % BuiltBlockID::COUNT;
                BuiltBlockID::from_repr_or_air(index)
            }
        })
        .collect();
    let g: [BuiltBlockID; CHUNK_LEN] = grid.try_into().unwrap();
    let dims: Dimensions = (FACTOR, FACTOR, FACTOR);

    let (culled_mesh, metadata) = mesh_grid(
        dims,
        &[],
        &g,
        breg.into_inner(),
        MeshingAlgorithm::Culling,
        None,
    )
    .unwrap();
    let culled_mesh_handle: Handle<Mesh> = meshes.add(culled_mesh.clone());

    simple_control::setup(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 20.0, 0.0),
    );

    let textures: Vec<Handle<Image>> = BLOCK_TEXTURES
        .iter()
        .map(|&path| asset_server.load(format!("images/blocks/{}", path)))
        .collect();

    commands.spawn((
        Mesh3d(culled_mesh_handle),
        MeshMaterial3d(materials.add(BindlessMaterial {
            uniforms: MaterialUniforms {
                texture_count: textures.len() as u32,
            },
            textures,
        })),
        Meshy {
            meta: metadata,
            grid: g,
        },
        Transform::from_xyz(-10.0, 0.0, -10.0),
    ));

    // Light
    // commands.spawn((
    //     Transform::from_rotation(Quat::from_euler(
    //         EulerRot::ZYX,
    //         0.0,
    //         1.0,
    //         -std::f32::consts::PI / 4.,
    //     )),
    //     DirectionalLight {
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     bevy::pbr::CascadeShadowConfigBuilder {
    //         first_cascade_far_bound: 200.0,
    //         maximum_distance: 400.0,
    //         ..default()
    //     }
    //     .build(),
    // ));
}

#[derive(Resource)]
struct BlockRegistry {
    grass: Mesh,
    dirt: Mesh,
    planks_oak: Mesh,
    wool_colored_orange: Mesh,
}

impl VoxelRegistry for BlockRegistry {
    type Voxel = BuiltBlockID;

    fn get_mesh(&self, voxel: &Self::Voxel) -> VoxelMesh<&Mesh> {
        match *voxel {
            BuiltBlockID::Air => VoxelMesh::Null,
            BuiltBlockID::Dirt => VoxelMesh::NormalCube(&self.dirt),
            BuiltBlockID::Grass => VoxelMesh::NormalCube(&self.grass),
            BuiltBlockID::PlanksOak => VoxelMesh::NormalCube(&self.planks_oak),
            BuiltBlockID::WoolColoredOrange => VoxelMesh::NormalCube(&self.wool_colored_orange),
        }
    }
    /// Important function that tells our Algorithm if the Voxel is "full", for example, the Air
    /// in minecraft is not "full", but it is still on the chunk data, to signal there is nothing.
    fn is_covering(&self, voxel: &Self::Voxel, _side: bevy_meshem::prelude::Face) -> bool {
        return *voxel != BuiltBlockID::Air;
    }
    /// The center of the Mesh, out mesh is defined in src/voxel_mesh.rs, just a constant.
    fn get_center(&self) -> [f32; 3] {
        return [0.0, 0.0, 0.0];
    }
    /// The dimensions of the Mesh, out mesh is defined in src/voxel_mesh.rs, just a constant.
    fn get_voxel_dimensions(&self) -> [f32; 3] {
        return [1.0, 1.0, 1.0];
    }
    /// The attributes we want to take from out voxels, note that using a lot of different
    /// attributes will likely lead to performance problems and unpredictable behaviour.
    /// We chose these 3 because they are very common, the algorithm does preserve UV data.
    fn all_attributes(&self) -> Vec<bevy::render::mesh::MeshVertexAttribute> {
        return vec![
            Mesh::ATTRIBUTE_POSITION,
            Mesh::ATTRIBUTE_UV_0,
            Mesh::ATTRIBUTE_NORMAL,
        ];
    }
}

/// System to add or break random voxels.
fn mesh_update(
    mut meshy: Query<&mut Meshy>,
    breg: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<&Mesh3d>,
    mut event_reader: EventReader<RegenerateMesh>,
) {
    for _ in event_reader.read() {
        let mesh = meshes
            .get_mut(mesh_query.single().unwrap())
            .expect("Couldn't get a mut ref to the mesh");

        let m = meshy.single_mut().unwrap().into_inner();
        let mut rng = rand::thread_rng();
        let choice = m.grid.iter().enumerate().choose(&mut rng).unwrap();

        let neighbors: [Option<BuiltBlockID>; 6] = {
            let mut r = [None; 6];
            for i in 0..6 {
                match get_neighbor(choice.0, bevy_meshem::prelude::Face::from(i), m.meta.dims) {
                    None => {}
                    Some(j) => r[i] = Some(m.grid[j]),
                }
            }
            r
        };

        let i = choice.0;
        let voxel = m.grid[i];
        m.meta.log(VoxelChange::Broken, i, voxel, neighbors);
        update_mesh(mesh, &mut m.meta, breg.into_inner());
        m.grid[i] = BuiltBlockID::Air;
        break;
    }
}

fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut Transform, With<Mesh3d>>,
    camera_query: Query<&GlobalTransform, (With<Camera>, Without<Mesh3d>)>,
    mut event_writer: EventWriter<viewer::wireframe::ToggleWireframe>,
    mut e: EventWriter<RegenerateMesh>,
    mut break_block_writer: EventWriter<BreakBlock>,
) {
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        event_writer.write_default();
    }
    if keyboard_input.pressed(KeyCode::KeyC) {
        e.write_default();
    }

    // 鼠标左键破坏方块
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(camera_transform) = camera_query.single() {
            break_block_writer.write(BreakBlock {
                ray_origin: camera_transform.translation(),
                ray_direction: camera_transform.forward().into(),
            });
        }
    }
}

// 添加射线投射破坏方块的系统
fn raycast_break_block(
    mut break_events: EventReader<BreakBlock>,
    mut meshy: Query<(&mut Meshy, &Transform)>,
    breg: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<&Mesh3d>,
) {
    let breg = breg.into_inner();

    for break_event in break_events.read() {
        let mesh = meshes
            .get_mut(mesh_query.single().unwrap())
            .expect("Couldn't get a mut ref to the mesh");

        if let Ok((mut m, mesh_transform)) = meshy.single_mut() {
            // 将射线转换到网格局部空间
            let inverse_transform = mesh_transform.compute_matrix().inverse();
            let local_origin = inverse_transform.transform_point3(break_event.ray_origin);
            let local_direction = inverse_transform
                .transform_vector3(break_event.ray_direction)
                .normalize();

            // 执行射线投射
            if let Some(hit_index) =
                raycast_voxel_grid(local_origin, local_direction, &m.grid, FACTOR)
            {
                let voxel = m.grid[hit_index];

                // 只破坏非空气方块
                if voxel != BuiltBlockID::Air {
                    // 获取邻居信息
                    let neighbors: [Option<BuiltBlockID>; 6] = {
                        let mut r = [None; 6];
                        for i in 0..6 {
                            match get_neighbor(
                                hit_index,
                                bevy_meshem::prelude::Face::from(i),
                                m.meta.dims,
                            ) {
                                None => {}
                                Some(j) => r[i] = Some(m.grid[j]),
                            }
                        }
                        r
                    };

                    // 记录变更并更新网格
                    m.meta.log(VoxelChange::Broken, hit_index, voxel, neighbors);
                    update_mesh(mesh, &mut m.meta, breg);
                    m.grid[hit_index] = BuiltBlockID::Air;
                }
            }
        }
    }
}

// 实现体素网格射线投射
fn raycast_voxel_grid(
    ray_origin: Vec3,
    ray_direction: Vec3,
    grid: &[BuiltBlockID; CHUNK_LEN],
    chunk_size: usize,
) -> Option<usize> {
    const MAX_DISTANCE: f32 = 10.0; // 最大破坏距离
    const STEP_SIZE: f32 = 0.1; // 射线步进大小

    let mut current_pos = ray_origin;
    let step = ray_direction * STEP_SIZE;
    let mut distance = 0.0;

    while distance < MAX_DISTANCE {
        // 检查当前位置是否在网格范围内
        if current_pos.x >= 0.0
            && current_pos.x < chunk_size as f32
            && current_pos.y >= 0.0
            && current_pos.y < chunk_size as f32
            && current_pos.z >= 0.0
            && current_pos.z < chunk_size as f32
        {
            // 转换为网格索引
            let x = current_pos.x.floor() as usize;
            let y = current_pos.y.floor() as usize;
            let z = current_pos.z.floor() as usize;

            // Notice: in meshem chunk, the z-position represents the depth. so we exchange y and z here.
            let index = x + z * chunk_size + y * chunk_size * chunk_size;

            // 检查索引是否有效
            if index < CHUNK_LEN {
                // 检查是否击中非空气方块
                if grid[index] != BuiltBlockID::Air {
                    return Some(index);
                }
            }
        }

        current_pos += step;
        distance += STEP_SIZE;
    }

    None
}
