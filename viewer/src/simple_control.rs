use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Component)]
pub struct PlayerCamera;

pub fn setup<M: Asset + TypePath>(
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<M>>,
    translation: Vec3,
    player_look_at: Option<Vec3>,
) {
    let (transform, initial_yaw, initial_pitch) = if let Some(look_at) = player_look_at {
        let transform = Transform::from_translation(translation).looking_at(look_at, Vec3::Y);
        let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
        (transform, yaw, pitch)
    } else {
        (Transform::from_translation(translation), 0.0, 0.0)
    };

    commands
        .spawn((
            Player {
                speed: 10.0,
                sensitivity: 0.003,
                yaw: initial_yaw,
                pitch: initial_pitch,
            },
            transform,
            Visibility::default(),
        ))
        .with_children(
            |parent: &mut bevy::ecs::relationship::RelatedSpawnerCommands<'_, ChildOf>| {
                parent.spawn((
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    Camera3d::default(),
                    Projection::Perspective(PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        near: 0.01,
                        far: 1000.0,
                        aspect_ratio: 1.0,
                    }),
                    PlayerCamera,
                ));
            },
        );
}

pub fn cursor_grab_system(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        if mouse.just_pressed(MouseButton::Left) {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }

        if key.just_pressed(KeyCode::Escape) {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

pub fn player_movement_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Player, &mut Transform), Without<PlayerCamera>>,
) {
    if let Ok((player, mut transform)) = player_query.single_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0.0, local_z.z);
        let right = Vec3::new(local_z.z, 0.0, -local_z.x);

        if keys.pressed(KeyCode::KeyW) {
            velocity += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            velocity -= forward;
        }
        if keys.pressed(KeyCode::KeyA) {
            velocity -= right;
        }
        if keys.pressed(KeyCode::KeyD) {
            velocity += right;
        }

        if keys.pressed(KeyCode::Space) {
            velocity += Vec3::Y;
        }
        if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            velocity -= Vec3::Y;
        }

        if velocity.length() > 0.0 {
            velocity = velocity.normalize();
            transform.translation += velocity * player.speed * time.delta_secs();
        }
    }
}

pub fn player_look_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut player_query: Query<(&mut Player, &mut Transform), Without<PlayerCamera>>,
    mut windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.single_mut() {
        if window.cursor_options.grab_mode == CursorGrabMode::Locked {
            if let Ok((mut player, mut transform)) = player_query.single_mut() {
                for ev in mouse_motion_events.read() {
                    player.yaw -= ev.delta.x * player.sensitivity;
                    player.pitch -= ev.delta.y * player.sensitivity;

                    let max_pitch = 85.0_f32.to_radians();
                    player.pitch = player.pitch.clamp(-max_pitch, max_pitch);

                    transform.rotation =
                        Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
                }
            }
        }
    }
}
