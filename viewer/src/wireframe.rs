use bevy::{
    asset::Assets, color::Alpha, ecs::{
        entity::Entity,
        event::{Event, EventReader},
        query::{With, Without},
        system::{Commands, Query, ResMut},
    }, pbr::{wireframe::Wireframe, StandardMaterial}, render::mesh::Mesh3d
};

#[derive(Event, Default)]
pub struct ToggleWireframe;

/// Function to toggle wireframe (seeing the vertices and indices of the mesh).
pub fn toggle_wireframe(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    with: Query<Entity, With<Wireframe>>,
    without: Query<Entity, (Without<Wireframe>, With<Mesh3d>)>,
    mut events: EventReader<ToggleWireframe>,
) {
    for _ in events.read() {
        if let Ok(ent) = with.single() {
            commands.entity(ent).remove::<Wireframe>();
            for (_, material) in materials.iter_mut() {
                material.base_color.set_alpha(1.0);
            }
        } else if let Ok(ent) = without.single() {
            commands.entity(ent).insert(Wireframe);
            for (_, material) in materials.iter_mut() {
                material.base_color.set_alpha(0.0);
            }
        }
    }
}
