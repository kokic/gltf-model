#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::EnumCount, strum_macros::FromRepr)]
pub enum BuiltBlockID {
    Air,
    Dirt,
    Grass,
    PlanksOak,
    WoolColoredOrange,
}

impl BuiltBlockID {
    pub fn from_repr_or_air(value: usize) -> Self {
        BuiltBlockID::from_repr(value).unwrap_or(BuiltBlockID::Air)
    }
}
