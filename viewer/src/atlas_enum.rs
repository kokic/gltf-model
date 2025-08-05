use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::system::{Res, ResMut},
    math::UVec2,
    ui::widget::ImageNode,
};
use bevy_image::{Image, TextureAtlas, TextureAtlasLayout};
use strum::IntoEnumIterator;

use crate::{gui_atlas::create_atlas_from_coords, region::Region};

pub trait AtlasEnum: IntoEnumIterator + Copy + Clone {
    fn texture_source() -> (&'static str, UVec2);
    fn get_region(&self) -> Region;
    fn get_index(&self) -> usize;

    fn to_regions() -> Vec<Region> {
        Self::iter().map(|icon| icon.get_region()).collect()
    }

    fn to_image_node(
        &self,
        texture_handle: Handle<Image>,
        atlas_layout_handle: Handle<TextureAtlasLayout>,
    ) -> ImageNode {
        ImageNode::from_atlas_image(
            texture_handle,
            TextureAtlas {
                index: self.get_index(),
                layout: atlas_layout_handle,
            },
        )
    }

    fn setup_atlas(
        asset_server: &Res<AssetServer>,
        texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
        let (texture_path, atlas_size) = Self::texture_source();
        create_atlas_from_coords(
            asset_server,
            texture_atlases,
            texture_path,
            atlas_size,
            &Self::to_regions(),
        )
    }
}
