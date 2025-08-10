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
pub struct TextureOverride(pub Option<Handle<Image>>);

pub fn observe(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
    override_query: Query<&TextureOverride>,
) {
    let Ok(texture_override) = override_query.get(trigger.target()) else {
        return;
    };

    if let Ok(direct_children) = children.get(trigger.target()) {
        for &child in direct_children.iter() {
            process_entity_recursive(
                child,
                &texture_override.0,
                &mut commands,
                &children,
                &mesh_materials,
                &mut asset_materials,
                &override_query,
            );
        }
    }
}

fn process_entity_recursive(
    entity: bevy::ecs::entity::Entity,
    texture: &Option<Handle<Image>>,
    commands: &mut Commands,
    children: &Query<&Children>,
    mesh_materials: &Query<&MeshMaterial3d<StandardMaterial>>,
    asset_materials: &mut ResMut<Assets<StandardMaterial>>,
    override_query: &Query<&TextureOverride>,
) {
    if override_query.get(entity).is_ok() {
        return;
    }

    if let Some(material) = mesh_materials
        .get(entity)
        .ok()
        .and_then(|id| asset_materials.get_mut(id.id()))
    {
        let mut new_material = material.clone();
        new_material.base_color_texture = texture.clone();
        new_material.alpha_mode = AlphaMode::Mask(0.5);

        commands
            .entity(entity)
            .insert(MeshMaterial3d(asset_materials.add(new_material)));
    }

    if let Ok(entity_children) = children.get(entity) {
        for &child in entity_children.iter() {
            process_entity_recursive(
                child,
                texture,
                commands,
                children,
                mesh_materials,
                asset_materials,
                override_query,
            );
        }
    }
}
