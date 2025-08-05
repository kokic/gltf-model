use bevy::render::mesh::Mesh;
use bevy_meshem::prelude::{
    generate_voxel_mesh,
    Face::{Back, Bottom, Forward, Left, Right, Top},
};

macro_rules! define_block_textures {
    ($($name:literal),* $(,)?) => {
        pub const BLOCK_TEXTURES: &[&str] = &[$($name),*];
        
        #[allow(unused_assignments)]
        pub fn get_texture_option(name: &str) -> Option<u32> {
            let mut index = 0;
            $(
                if name == $name {
                    return Some(index);
                }
                index += 1;
            )*
            None
        }

        /// Return `missing_tile.png` index if not found
        pub fn get_texture(name: &str) -> u32 {
            get_texture_option(name).unwrap_or(0)
        }
    };
}

define_block_textures!(
    "missing_tile.png", 
    "brick.png",
    "dirt.png",
    "grass_carried.png",
    "grass_side_carried.png", 
    "planks_oak.png",
    "wool_colored_orange.png",
);

pub const MAX_BLOCK_TEXTURE_COUNT: usize = BLOCK_TEXTURES.len();

pub type BlockTextureIndex = u32;

/// All = `side`
pub fn isotropic_mesh(side: BlockTextureIndex) -> Mesh {
    return simple_cubic_mesh(side, side, side, side, side, side);
}

/// Top = Bottom = `axial`, Others = `side`
pub fn axial_mesh(axial: BlockTextureIndex, side: BlockTextureIndex) -> Mesh {
    return simple_cubic_mesh(axial, axial, side, side, side, side);
}

/// Top = `top`, Bottom = `bottom`, Others = `side`
pub fn top_bottom_mesh(
    top: BlockTextureIndex,
    bottom: BlockTextureIndex,
    side: BlockTextureIndex,
) -> Mesh {
    return simple_cubic_mesh(top, bottom, side, side, side, side);
}

pub fn simple_cubic_mesh(
    top: BlockTextureIndex,
    bottom: BlockTextureIndex,
    right: BlockTextureIndex,
    left: BlockTextureIndex,
    back: BlockTextureIndex,
    forward: BlockTextureIndex,
) -> Mesh {
    return generate_voxel_mesh(
        [1.0, 1.0, 1.0],
        [MAX_BLOCK_TEXTURE_COUNT as u32, 1],
        [
            (Top, [top, 0]),
            (Bottom, [bottom, 0]),
            (Right, [right, 0]),
            (Left, [left, 0]),
            (Back, [back, 0]),
            (Forward, [forward, 0]),
        ],
        [0.0, 0.0, 0.0],
        0.0,
        Some(0.8),
        1.0,
    );
}
