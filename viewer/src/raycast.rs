use bevy::{
    color::Color,
    ecs::{
        query::With,
        resource::Resource,
        system::{Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    input::{mouse::MouseButton, ButtonInput},
    math::{Vec2, Vec3},
    render::camera::Camera,
    transform::components::GlobalTransform,
    window::Window,
};

use crate::{
    block::BuiltBlockID,
    chunk::{Chunk, World, WorldPos},
    simple_control::PlayerCamera,
};

#[derive(Resource, Default)]
pub struct RaycastDebugInfo {
    pub last_hit: Option<WorldPos>,
    pub ray_origin: Vec3,
    pub ray_direction: Vec3,
    pub precise_hit_point: Option<Vec3>,
    pub hit_face_normal: Option<Vec3>,
}

pub fn update_outline_box(
    camera_query: Query<(&GlobalTransform, &Camera), With<PlayerCamera>>,
    mut gizmos: Gizmos,
    mouse_input: Res<ButtonInput<MouseButton>>,
    world: Res<World>,
    chunks: Query<&mut Chunk>,
    windows: Query<&Window>,
    mut debug_info: ResMut<RaycastDebugInfo>,
) {
    if let Ok((camera_transform, camera)) = camera_query.single() {
        if let Ok(window) = windows.single() {
            if let Some((ray_origin, ray_direction)) =
                get_camera_ray(camera_transform, camera, window)
            {
                debug_info.ray_origin = ray_origin;
                debug_info.ray_direction = ray_direction;

                if let Some(hit) = precise_minecraft_raycast(
                    ray_origin,
                    ray_direction,
                    &world,
                    &chunks,
                    20.0,
                ) {
                    debug_info.last_hit = Some(hit.position);
                    debug_info.precise_hit_point = Some(hit.hit_point);
                    debug_info.hit_face_normal = Some(hit.face_normal);

                    draw_precise_block_outline(
                        &mut gizmos,
                        hit.position,
                        Color::linear_rgb(1.0, 1.0, 0.0),
                    );
                } else {
                    debug_info.last_hit = None;
                    debug_info.precise_hit_point = None;
                    debug_info.hit_face_normal = None;
                }
            }
        }
    }
}

pub fn voxel_raycast(
    ray_origin: Vec3,
    ray_direction: Vec3,
    world: &World,
    chunks: &Query<&mut Chunk>,
) -> Option<WorldPos> {
    precise_minecraft_raycast(ray_origin, ray_direction, world, chunks, 20.0)
        .map(|hit| hit.position)
}

fn draw_precise_block_outline(gizmos: &mut Gizmos, block_pos: WorldPos, color: Color) {
    let min = Vec3::new(
        block_pos.x as f32 - 0.5,
        block_pos.y as f32 - 0.5,
        block_pos.z as f32 - 0.5,
    );
    let max = min + Vec3::ONE;

    gizmos.line(
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        color,
    );
    gizmos.line(
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, min.y, max.z),
        color,
    );
    gizmos.line(
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, min.y, max.z),
        color,
    );
    gizmos.line(
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(min.x, min.y, min.z),
        color,
    );

    gizmos.line(
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        color,
    );
    gizmos.line(
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(max.x, max.y, max.z),
        color,
    );
    gizmos.line(
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        color,
    );
    gizmos.line(
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(min.x, max.y, min.z),
        color,
    );

    gizmos.line(
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        color,
    );
    gizmos.line(
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        color,
    );
    gizmos.line(
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(max.x, max.y, max.z),
        color,
    );
    gizmos.line(
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        color,
    );
}

#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub position: WorldPos,
    pub hit_point: Vec3,
    pub face_normal: Vec3,
    pub distance: f32,
    pub face: HitFace,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitFace {
    None,
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

pub fn get_camera_ray(
    camera_transform: &GlobalTransform,
    camera: &Camera,
    window: &Window,
) -> Option<(Vec3, Vec3)> {
    let screen_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);

    if let Ok(viewport_ray) = camera.viewport_to_world(camera_transform, screen_center) {
        let ray_origin = camera_transform.translation();
        let ray_direction = viewport_ray.direction.normalize();
        return Some((ray_origin, ray_direction));
    }

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().normalize();

    Some((ray_origin, ray_direction))
}

pub fn precise_minecraft_raycast(
    ray_origin: Vec3,
    ray_direction: Vec3,
    world: &World,
    chunks: &Query<&mut Chunk>,
    max_distance: f32,
) -> Option<RaycastHit> {
    let origin = ray_origin.as_dvec3();
    let direction = ray_direction.normalize().as_dvec3();

    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    let start_pos = WorldPos { x, y, z };
    if world.get_block(start_pos, chunks) != BuiltBlockID::Air {
        let hit_point = calculate_precise_hit_point(ray_origin, ray_direction, start_pos);
        let face_normal = calculate_hit_face_normal(ray_origin, ray_direction, start_pos);
        let face = determine_hit_face(ray_origin, ray_direction, start_pos);
        
        return Some(RaycastHit {
            position: start_pos,
            hit_point,
            face_normal,
            distance: 0.0,
            face,
        });
    }

    let step_x = if direction.x > 0.0 { 1 } else { -1 };
    let step_y = if direction.y > 0.0 { 1 } else { -1 };
    let step_z = if direction.z > 0.0 { 1 } else { -1 };

    let epsilon = 1e-15;
    let delta_x = if direction.x.abs() < epsilon {
        f64::INFINITY
    } else {
        (1.0 / direction.x).abs()
    };
    let delta_y = if direction.y.abs() < epsilon {
        f64::INFINITY
    } else {
        (1.0 / direction.y).abs()
    };
    let delta_z = if direction.z.abs() < epsilon {
        f64::INFINITY
    } else {
        (1.0 / direction.z).abs()
    };

    let mut max_x = if direction.x.abs() < epsilon {
        f64::INFINITY
    } else if direction.x > 0.0 {
        ((x + 1) as f64 - origin.x) / direction.x
    } else {
        (origin.x - x as f64) / (-direction.x)
    };

    let mut max_y = if direction.y.abs() < epsilon {
        f64::INFINITY
    } else if direction.y > 0.0 {
        ((y + 1) as f64 - origin.y) / direction.y
    } else {
        (origin.y - y as f64) / (-direction.y)
    };

    let mut max_z = if direction.z.abs() < epsilon {
        f64::INFINITY
    } else if direction.z > 0.0 {
        ((z + 1) as f64 - origin.z) / direction.z
    } else {
        (origin.z - z as f64) / (-direction.z)
    };

    let mut distance = 0.0;
    let max_distance_f64 = max_distance as f64;
    
    let max_iterations = (max_distance * 2.0) as i32 + 100;
    let mut iteration_count = 0;
    
    while distance < max_distance_f64 && iteration_count < max_iterations {
        iteration_count += 1;
        
        if max_x < max_y && max_x < max_z {
            x += step_x;
            distance = max_x;
            max_x += delta_x;
        } else if max_y < max_z {
            y += step_y;
            distance = max_y;
            max_y += delta_y;
        } else {
            z += step_z;
            distance = max_z;
            max_z += delta_z;
        }

        let current_pos = WorldPos { x, y, z };

        if world.get_block(current_pos, chunks) != BuiltBlockID::Air {
            let hit_point = calculate_precise_hit_point(ray_origin, ray_direction, current_pos);
            let face_normal = calculate_hit_face_normal(ray_origin, ray_direction, current_pos);
            let face = determine_hit_face(ray_origin, ray_direction, current_pos);

            return Some(RaycastHit {
                position: current_pos,
                hit_point,
                face_normal,
                distance: distance as f32,
                face,
            });
        }
    }

    None
}

fn calculate_precise_hit_point(ray_origin: Vec3, ray_direction: Vec3, block_pos: WorldPos) -> Vec3 {
    let block_min = Vec3::new(
        block_pos.x as f32 - 0.5,
        block_pos.y as f32 - 0.5,
        block_pos.z as f32 - 0.5,
    );
    let block_max = block_min + Vec3::ONE;

    let direction = ray_direction.normalize();
    
    let epsilon = 1e-6f32;

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;
    let mut hit_face_axis = 0;
    let mut hit_face_sign = 1.0;

    for i in 0..3 {
        let ray_origin_i = ray_origin[i];
        let ray_dir_i = direction[i];
        let box_min_i = block_min[i];
        let box_max_i = block_max[i];

        if ray_dir_i.abs() < epsilon {
            if ray_origin_i < box_min_i || ray_origin_i > box_max_i {
                return ray_origin;
            }
        } else {
            let inv_dir = 1.0 / ray_dir_i;
            let mut t1 = (box_min_i - ray_origin_i) * inv_dir;
            let mut t2 = (box_max_i - ray_origin_i) * inv_dir;

            let mut face_sign = -1.0;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
                face_sign = 1.0;
            }

            if t1 > t_min {
                t_min = t1;
                hit_face_axis = i;
                hit_face_sign = face_sign;
            }
            t_max = t_max.min(t2);

            if t_min > t_max {
                return ray_origin;
            }
        }
    }

    let t = if t_min >= 0.0 { t_min } else { t_max };
    if t >= 0.0 {
        ray_origin + direction * t
    } else {
        let block_center = Vec3::new(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);
        let offset = ray_origin - block_center;
        
        let abs_offset = offset.abs();
        if abs_offset.x >= abs_offset.y && abs_offset.x >= abs_offset.z {
            Vec3::new(block_center.x + 0.5 * offset.x.signum(), ray_origin.y, ray_origin.z)
        } else if abs_offset.y >= abs_offset.z {
            Vec3::new(ray_origin.x, block_center.y + 0.5 * offset.y.signum(), ray_origin.z)
        } else {
            Vec3::new(ray_origin.x, ray_origin.y, block_center.z + 0.5 * offset.z.signum())
        }
    }
}

fn calculate_hit_face_normal(ray_origin: Vec3, ray_direction: Vec3, block_pos: WorldPos) -> Vec3 {
    let hit_point = calculate_precise_hit_point(ray_origin, ray_direction, block_pos);
    let block_center = Vec3::new(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);

    let offset = hit_point - block_center;
    let tolerance = 0.001;

    if (offset.x.abs() - 0.5).abs() < tolerance {
        Vec3::new(offset.x.signum(), 0.0, 0.0)
    } else if (offset.y.abs() - 0.5).abs() < tolerance {
        Vec3::new(0.0, offset.y.signum(), 0.0)
    } else if (offset.z.abs() - 0.5).abs() < tolerance {
        Vec3::new(0.0, 0.0, offset.z.signum())
    } else {
        Vec3::ZERO
    }
}

fn determine_hit_face(ray_origin: Vec3, ray_direction: Vec3, block_pos: WorldPos) -> HitFace {
    let normal = calculate_hit_face_normal(ray_origin, ray_direction, block_pos);

    if normal.x > 0.5 {
        HitFace::PosX
    } else if normal.x < -0.5 {
        HitFace::NegX
    } else if normal.y > 0.5 {
        HitFace::PosY
    } else if normal.y < -0.5 {
        HitFace::NegY
    } else if normal.z > 0.5 {
        HitFace::PosZ
    } else if normal.z < -0.5 {
        HitFace::NegZ
    } else {
        HitFace::None
    }
}

pub fn get_adjacent_empty_position(
    hit: &RaycastHit,
    world: &World,
    chunks: &Query<&mut Chunk>,
) -> Option<WorldPos> {
    println!("击中位置: {:?}", hit.position);
    println!("击中面: {:?}", hit.face);
    println!("面法向量: {:?}", hit.face_normal);
    
    let offset = match hit.face {
        HitFace::PosX => WorldPos { x: 1, y: 0, z: 0 },
        HitFace::NegX => WorldPos { x: -1, y: 0, z: 0 },
        HitFace::PosY => WorldPos { x: 0, y: 1, z: 0 },
        HitFace::NegY => WorldPos { x: 0, y: -1, z: 0 },
        HitFace::PosZ => WorldPos { x: 0, y: 0, z: 1 },
        HitFace::NegZ => WorldPos { x: 0, y: 0, z: -1 },
        HitFace::None => {
            println!("错误：击中面为None");
            return None;
        }
    };

    let adjacent_pos = WorldPos {
        x: hit.position.x + offset.x,
        y: hit.position.y + offset.y,
        z: hit.position.z + offset.z,
    };

    println!("目标放置位置: {:?}", adjacent_pos);
    
    let target_chunk_pos = adjacent_pos.to_chunk_pos();
    let hit_chunk_pos = hit.position.to_chunk_pos();
    
    println!("击中方块的chunk: {:?}", hit_chunk_pos);
    println!("目标位置的chunk: {:?}", target_chunk_pos);

    let block_at_pos = world.get_block(adjacent_pos, chunks);
    println!("目标位置的方块类型: {:?}", block_at_pos);
    
    if block_at_pos == BuiltBlockID::Air {
        println!("放置成功！");
        Some(adjacent_pos)
    } else {
        println!("放置失败：位置被 {:?} 占用", block_at_pos);
        None
    }
}
