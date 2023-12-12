use crate::geo::Mesh;

/// Flat image lithophane generator
pub mod flat_image;

pub trait LithophaneGenerator {
    fn generate(self) -> Mesh;
}
