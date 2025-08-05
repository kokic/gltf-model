use bevy::{
    color::Color,
    ecs::system::Commands,
    math::{EulerRot, Quat},
    pbr::{AmbientLight, DirectionalLight},
    transform::components::Transform,
    utils::default,
};

pub fn setup_simple_light(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.95, 0.95, 1.0), 
        brightness: 1000.0,
        ..default()
    });
    
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.95, 0.8),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 3.0,
            std::f32::consts::PI / 4.0,
            0.0,
        )),
        bevy::pbr::CascadeShadowConfigBuilder {
            first_cascade_far_bound: 50.0,
            maximum_distance: 200.0,
            ..default()
        }
        .build(),
    ));

}
