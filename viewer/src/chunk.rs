use std::collections::HashMap;

use bevy::{
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    pbr::MeshMaterial3d,
    platform::collections::HashSet,
    render::mesh::{Mesh, Mesh3d},
    transform::components::Transform,
};
use bevy_image::Image;
use bevy_meshem::{
    prelude::{mesh_grid, MeshMD, MeshingAlgorithm},
    Dimensions,
};

use crate::{
    bindless_material::{BindlessMaterial, MaterialUniforms},
    block::{BlockRegistry, BuiltBlockID},
};

const SIZE: usize = 8;
const CHUNK_LEN: usize = SIZE * SIZE * SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPos {
    pub fn to_chunk_pos(self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(SIZE as i32),
            y: self.y.div_euclid(SIZE as i32),
            z: self.z.div_euclid(SIZE as i32),
        }
    }

    pub fn to_local_pos(self) -> (usize, usize, usize) {
        (
            self.x.rem_euclid(SIZE as i32) as usize,
            self.y.rem_euclid(SIZE as i32) as usize,
            self.z.rem_euclid(SIZE as i32) as usize,
        )
    }

    pub fn to_local_index(self) -> usize {
        let (x, y, z) = self.to_local_pos();
        // Note: y corresponds to the height within the chunk
        x + z * SIZE + y * SIZE * SIZE
    }
}

impl ChunkPos {
    pub fn to_world_pos(
        self,
        local_x: usize,
        local_y: usize,
        local_z: usize,
    ) -> WorldPos {
        WorldPos {
            x: self.x * SIZE as i32 + local_x as i32,
            y: self.y * SIZE as i32 + local_y as i32,
            z: self.z * SIZE as i32 + local_z as i32,
        }
    }
}

#[derive(Component)]
pub struct Chunk {
    #[allow(dead_code)]
    pos: ChunkPos,
    meta: MeshMD<BuiltBlockID>,
    grid: [BuiltBlockID; CHUNK_LEN],
    dirty: bool,
}

#[derive(Resource)]
pub struct World {
    chunks: HashMap<ChunkPos, Entity>,
    loaded_chunks: HashSet<ChunkPos>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            loaded_chunks: HashSet::new(),
        }
    }

    /// immediately set a block
    pub fn set_block(
        &mut self,
        world_pos: WorldPos,
        block: BuiltBlockID,
        commands: &mut Commands,
        chunks: &mut Query<&mut Chunk>,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<BindlessMaterial>>,
        breg: &BlockRegistry,
        block_textures: &[Handle<Image>],
    ) -> bool {
        let chunk_pos = world_pos.to_chunk_pos();

        if !self.chunks.contains_key(&chunk_pos) {
            self.create_chunk_now(
                commands,
                meshes,
                materials,
                breg,
                chunk_pos,
                block_textures,
            );
        }

        self.set_block_in_chunk(world_pos, block, chunks)
    }

    pub fn create_chunk_now(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<BindlessMaterial>>,
        breg: &BlockRegistry,
        chunk_pos: ChunkPos,
        block_textures: &[Handle<Image>],
    ) {
        spawn_chunk(
            self,
            commands,
            meshes,
            materials,
            breg,
            chunk_pos,
            block_textures,
            None,
        );
    }

    pub fn set_block_in_chunk(
        &mut self,
        world_pos: WorldPos,
        block: BuiltBlockID,
        chunks: &mut Query<&mut Chunk>,
    ) -> bool {
        let chunk_pos = world_pos.to_chunk_pos();
        if let Some(&chunk_entity) = self.chunks.get(&chunk_pos) {
            if let Ok(mut chunk) = chunks.get_mut(chunk_entity) {
                let index = world_pos.to_local_index();
                chunk.grid[index] = block;
                chunk.dirty = true;
                return true;
            }
        }
        false
    }

    pub fn get_block(
        &self,
        world_pos: WorldPos,
        chunks: &Query<&mut Chunk>,
    ) -> BuiltBlockID {
        let chunk_pos = world_pos.to_chunk_pos();
        if let Some(&chunk_entity) = self.chunks.get(&chunk_pos) {
            if let Ok(chunk) = chunks.get(chunk_entity) {
                let index = world_pos.to_local_index();
                return chunk.grid[index];
            }
        }
        BuiltBlockID::Air
    }
}

#[derive(Event, Default)]
pub struct RegenerateMesh;

#[derive(Event)]
pub struct SetBlockEvent {
    pub world_pos: WorldPos,
    pub block: BuiltBlockID,
}

#[derive(Event)]
pub struct GetBlockEvent {
    pub world_pos: WorldPos,
    pub response_sender: Option<std::sync::mpsc::Sender<BuiltBlockID>>,
}

#[derive(Resource)]
pub struct BlockTextures(pub Vec<Handle<Image>>);

pub fn handle_set_block_events(
    mut set_block_events: EventReader<SetBlockEvent>,
    mut world: ResMut<World>,
    mut commands: Commands,
    mut chunks: Query<&mut Chunk>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BindlessMaterial>>,
    breg: Res<BlockRegistry>,
    block_textures: Res<BlockTextures>,
) {
    for event in set_block_events.read() {
        world.set_block(
            event.world_pos,
            event.block,
            &mut commands,
            &mut chunks,
            &mut meshes,
            &mut materials,
            &breg,
            &block_textures.0,
        );
    }
}

pub fn update_dirty_chunks(
    mut chunks: Query<(&mut Chunk, &Mesh3d)>,
    breg: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let breg = breg.into_inner();

    for (mut chunk, mesh3d) in chunks.iter_mut() {
        if chunk.dirty {
            if let Some(mesh) = meshes.get_mut(&mesh3d.0) {
                let (new_mesh, new_metadata) = mesh_grid(
                    (SIZE, SIZE, SIZE),
                    &[],
                    &chunk.grid,
                    breg,
                    MeshingAlgorithm::Culling,
                    None,
                )
                .unwrap();

                *mesh = new_mesh;
                chunk.meta = new_metadata;
                chunk.dirty = false;
            }
        }
    }
}

pub fn spawn_chunk(
    world: &mut World,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<BindlessMaterial>>,
    breg: &BlockRegistry,
    chunk_pos: ChunkPos,
    textures: &[Handle<Image>],
    fill_with: Option<BuiltBlockID>,
) {
    let fill_block = fill_with.unwrap_or(BuiltBlockID::Air);
    let grid = vec![fill_block; CHUNK_LEN];
    let grid_array: [BuiltBlockID; CHUNK_LEN] = grid.try_into().unwrap();
    let dims: Dimensions = (SIZE, SIZE, SIZE);

    let (culled_mesh, metadata) = mesh_grid(
        dims,
        &[],
        &grid_array,
        breg,
        MeshingAlgorithm::Culling,
        None,
    )
    .unwrap();

    let culled_mesh_handle: Handle<Mesh> = meshes.add(culled_mesh);

    let chunk_entity = commands
        .spawn((
            Mesh3d(culled_mesh_handle),
            MeshMaterial3d(materials.add(BindlessMaterial {
                uniforms: MaterialUniforms {
                    texture_count: textures.len() as u32,
                },
                textures: textures.to_vec(),
            })),
            Chunk {
                pos: chunk_pos,
                meta: metadata,
                grid: grid_array,
                dirty: false,
            },
            Transform::from_xyz(
                chunk_pos.x as f32 * SIZE as f32,
                chunk_pos.y as f32 * SIZE as f32,
                chunk_pos.z as f32 * SIZE as f32,
            ),
        ))
        .id();

    world.chunks.insert(chunk_pos, chunk_entity);
    world.loaded_chunks.insert(chunk_pos);
}
