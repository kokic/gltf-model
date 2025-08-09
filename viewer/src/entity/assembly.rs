use bevy::{math::Vec3, transform::components::Transform};

pub type Pixel = f32;

pub trait Localizable<Joint> {
    fn position() -> Vec3;
}

pub struct Head;
pub struct Body;

pub fn get_assembly_transform<
    Carrier: Localizable<CarrierJoint>,
    CarrierJoint,
    Carried: Localizable<CarriedJoint>,
    CarriedJoint,
>() -> Transform {
    let carrier_position = <Carrier as Localizable<CarrierJoint>>::position();
    let carried_position = <Carried as Localizable<CarriedJoint>>::position();
    let assembly = (carrier_position - carried_position) / 16.0;
    Transform::from_xyz(assembly.x, assembly.y, assembly.z)
}
