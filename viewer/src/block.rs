use bevy::{ecs::resource::Resource, render::mesh::Mesh};
use bevy_meshem::{VoxelMesh, VoxelRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::EnumCount, strum_macros::FromRepr)]
pub enum BuiltBlockID {
    Air,
    Brick, 
    Dirt,
    Grass,
    PlanksOak,
    WoolColoredOrange,
}

impl BuiltBlockID {
    pub fn from_repr_or_air(value: usize) -> Self {
        BuiltBlockID::from_repr(value).unwrap_or(BuiltBlockID::Air)
    }
}

#[derive(Resource)]
pub struct BlockRegistry {
    pub grass: Mesh,
    pub brick: Mesh,
    pub dirt: Mesh,
    pub planks_oak: Mesh,
    pub wool_colored_orange: Mesh,
}

impl VoxelRegistry for BlockRegistry {
    type Voxel = BuiltBlockID;

    fn get_mesh(&self, voxel: &Self::Voxel) -> VoxelMesh<&Mesh> {
        match *voxel {
            BuiltBlockID::Air => VoxelMesh::Null,
            BuiltBlockID::Brick => VoxelMesh::NormalCube(&self.brick),
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

