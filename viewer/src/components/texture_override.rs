use bevy::{
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        hierarchy::Children,
        observer::Trigger,
        system::{Commands, Query, ResMut},
    },
    pbr::{MeshMaterial3d, StandardMaterial},
    render::alpha::AlphaMode,
    scene::SceneInstanceReady,
};
use bevy_image::Image;

#[derive(Component)]
pub struct TextureOverride(pub Handle<Image>);

pub fn observe(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
    texture_override: Query<&TextureOverride>,
) {
    let Ok(texture_override) = texture_override.get(trigger.target()) else {
        return;
    };

    for descendant in children.iter_descendants(trigger.target()) {
        if let Some(material) = mesh_materials
            .get(descendant)
            .ok()
            .and_then(|id| asset_materials.get_mut(id.id()))
        {
            let mut new_material = material.clone();
            new_material.base_color_texture = Some(texture_override.0.clone());
            new_material.alpha_mode = AlphaMode::Mask(0.5);
            new_material.depth_bias = 0.1;

            commands
                .entity(descendant)
                .insert(MeshMaterial3d(asset_materials.add(new_material)));
        }
    }
}
