use bevy::{
    color::Color, ecs::{component::Component, query::With, system::{Commands, Query}}, render::camera::Camera, text::{TextColor, TextFont}, transform::components::GlobalTransform, ui::{
        widget::Text, AlignItems, FlexDirection, JustifyContent, Node, PositionType, UiRect, Val,
    }
};

#[derive(Component)]
pub struct CoordinateDisplay;

pub fn setup(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(20.0)),
            ..bevy::utils::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("X: 0.0, Y: 0.0, Z: 0.0"),
                TextFont {
                    font_size: 20.0,
                    ..bevy::utils::default()
                },
                TextColor(Color::WHITE),
                CoordinateDisplay,
            ));
        });
}

pub fn update_coordinate_display(
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut text_query: Query<&mut Text, With<CoordinateDisplay>>,
) {
    if let Ok(camera_transform) = camera_query.single() {
        if let Ok(mut text) = text_query.single_mut() {
            let pos = camera_transform.translation();
            text.0 = format!("X: {:.1}, Y: {:.1}, Z: {:.1}", pos.x, pos.y, pos.z);
        }
    }
}
