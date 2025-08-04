use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use bevy_meshem::{
    prelude::{mesh_grid, update_mesh, MeshMD, MeshingAlgorithm, VoxelChange},
    util::get_neighbor,
    Dimensions, VoxelMesh, VoxelRegistry,
};
use rand::prelude::*;

use viewer::bindless_material::{BindlessMaterial, MaterialUniforms};
use viewer::block::BuiltBlockID;
use viewer::built_block_mesh::{get_texture, isotropic_mesh, top_bottom_mesh, BLOCK_TEXTURES};

use viewer::gpu_fsc::GpuFeatureSupportChecker;
use viewer::simple_control;

const FACTOR: usize = 8;
const CHUNK_LEN: usize = FACTOR * FACTOR * FACTOR;

pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
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

    app.add_systems(Startup, setup)
        .add_systems(Update, (input_handler, toggle_wireframe, mesh_update))
        .add_systems(
            Update,
            (
                simple_control::cursor_grab_system,
                simple_control::player_movement_system,
                simple_control::player_look_system,
            ),
        );

    app.add_event::<ToggleWireframe>()
        .add_event::<RegenerateMesh>();

    app.run();
}

#[derive(Component)]
struct Meshy {
    meta: MeshMD<BuiltBlockID>,
    grid: [BuiltBlockID; CHUNK_LEN],
}

#[derive(Event, Default)]
struct ToggleWireframe;

#[derive(Event, Default)]
struct RegenerateMesh;

/// Setting up everything to showcase the mesh.
fn setup(
    breg: Res<BlockRegistry>,
    mut commands: Commands,
    mut materials: ResMut<Assets<BindlessMaterial>>,
    // wireframe_config: ResMut<WireframeConfig>,
    // mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let mut grid: Vec<BuiltBlockID> = vec![BuiltBlockID::Air; CHUNK_LEN];
    grid = grid
        .iter_mut()
        .enumerate()
        .map(|(i, _x)| {
            let x = i % FACTOR;
            let y = (i / FACTOR) % FACTOR;
            let z = i / (FACTOR * FACTOR);

            if z >= FACTOR - 1 {
                BuiltBlockID::Grass
            } else if z >= FACTOR - 4 {
                BuiltBlockID::Dirt
            } else {
                let index = (x + y) % 3 + 3;
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
        Vec3::new(0.0, 10.0, 0.0),
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
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::PI / 4.,
        )),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        bevy::pbr::CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));
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
    mut query: Query<&mut Transform, With<Mesh3d>>,
    // time: Res<Time>,
    mut event_writer: EventWriter<ToggleWireframe>,
    mut e: EventWriter<RegenerateMesh>,
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
}

/// Function to toggle wireframe (seeing the vertices and indices of the mesh).
fn toggle_wireframe(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    with: Query<Entity, With<Wireframe>>,
    without: Query<Entity, (Without<Wireframe>, With<Mesh3d>)>,
    mut events: EventReader<ToggleWireframe>,
) {
    for _ in events.read() {
        if let Ok(ent) = with.single() {
            commands.entity(ent).remove::<Wireframe>();
            for (_, material) in materials.iter_mut() {
                material.base_color.set_alpha(1.0);
            }
        } else if let Ok(ent) = without.single() {
            commands.entity(ent).insert(Wireframe);
            for (_, material) in materials.iter_mut() {
                material.base_color.set_alpha(0.0);
            }
        }
    }
}
