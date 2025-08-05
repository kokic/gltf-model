use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::system::{Res, ResMut},
    math::{URect, UVec2},
};
use bevy_image::{Image, TextureAtlasLayout};

use crate::{atlas_enum::AtlasEnum, region::Region};

#[derive(strum_macros::EnumIter, Copy, Clone)]
pub enum IconsAtlas {
    Crosshair,
}

impl AtlasEnum for IconsAtlas {
    fn get_region(&self) -> Region {
        match self {
            IconsAtlas::Crosshair => Region {
                x: 0,
                y: 0,
                w: 16,
                h: 16,
            },
        }
    }

    fn texture_source() -> (&'static str, UVec2) {
        ("images/gui/icons.png", UVec2::new(256, 256))
    }

    fn get_index(&self) -> usize {
        *self as usize
    }
}

pub fn create_atlas_from_coords(
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    texture_path: &str,
    atlas_size: UVec2,
    regions: &[Region],
) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
    let texture_handle: Handle<Image> = asset_server.load(texture_path);
    let mut atlas_layout = TextureAtlasLayout::new_empty(atlas_size);

    for region in regions {
        atlas_layout.add_texture(URect {
            min: UVec2::new(region.x, region.y),
            max: UVec2::new(region.x + region.w, region.y + region.h),
        });
    }

    let atlas_layout_handle = texture_atlases.add(atlas_layout);
    (texture_handle, atlas_layout_handle)
}
