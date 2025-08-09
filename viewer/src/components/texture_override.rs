use bevy::{prelude::*, scene::SceneInstanceReady};

#[derive(Component)]
pub struct TextureOverride(pub Handle<Image>);

#[derive(Component)]
pub struct PreserveOriginalMaterial;

pub fn observe(
    trigger: Trigger<SceneInstanceReady>,
    texture_override_query: Query<&TextureOverride>,
    preserve_query: Query<&PreserveOriginalMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_query: Query<&MeshMaterial3d<StandardMaterial>>,
    children_query: Query<&Children>,
    names: Query<&Name>,
) {
    let Ok(texture_override) = texture_override_query.get(trigger.target()) else {
        return;
    };

    debug!("Processing TextureOverride for entity {:?}", trigger.target());

    apply_texture_smart(
        trigger.target(),
        &texture_override.0,
        &mut materials,
        &mesh_query,
        &children_query,
        &texture_override_query,
        &preserve_query,
        &names,
        true,
    );
}

fn apply_texture_smart(
    entity: Entity,
    texture: &Handle<Image>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mesh_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    children_query: &Query<&Children>,
    texture_override_query: &Query<&TextureOverride>,
    preserve_query: &Query<&PreserveOriginalMaterial>,
    names: &Query<&Name>,
    is_root_entity: bool,
) {
    if preserve_query.contains(entity) {
        debug!("Preserving original material for entity {:?}", entity);
        return;
    }

    if !is_root_entity && texture_override_query.contains(entity) {
        debug!("Skipping entity {:?} - has own TextureOverride", entity);
        return;
    }

    if let Ok(mesh_material) = mesh_query.get(entity) {
        if let Some(material) = materials.get_mut(&mesh_material.0) {
            material.base_color_texture = Some(texture.clone());
            debug!("Applied texture to entity {:?}", entity);
        }
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            apply_texture_smart(
                child,
                texture,
                materials,
                mesh_query,
                children_query,
                texture_override_query,
                preserve_query,
                names,
                false,
            );
        }
    }
}
