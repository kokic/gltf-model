use bevy::{
    asset::Handle,
    ecs::{component::Component, system::Commands},
    ui::{AlignItems, JustifyContent, Node, PositionType, Val},
};
use bevy_image::{Image, TextureAtlasLayout};

use crate::{atlas_enum::AtlasEnum, gui_atlas};

#[derive(Component)]
struct Crosshair;

pub fn setup(
    commands: &mut Commands,
    texture_handle: Handle<Image>,
    atlas_layout_handle: Handle<TextureAtlasLayout>,
) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..bevy::utils::default()
        })
        .with_children(|parent| {
            parent.spawn((
                gui_atlas::IconsAtlas::Crosshair
                    .to_image_node(texture_handle, atlas_layout_handle),
                Crosshair,
            ));
        });
}
