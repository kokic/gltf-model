use bevy::asset::{AssetServer, Handle};
use bevy_image::Image;

use crate::model::ModelData;

type ItemID = &'static str;

pub struct DroppingItem {
    pub item: ItemID,
}

impl ModelData for DroppingItem {
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
