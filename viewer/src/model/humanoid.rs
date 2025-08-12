use bevy::math::Vec3;

use crate::model::assembly::{Body, Head, Localizable};

pub struct Humanoid;

impl Localizable<Head> for Humanoid {
    fn position() -> Vec3 {
        Vec3::new(-4.0, 24.0, -4.0)
    }
}

impl Localizable<Body> for Humanoid {
    fn position() -> Vec3 {
        Vec3::new(-4.0, 24.0, -4.0)
    }
}
