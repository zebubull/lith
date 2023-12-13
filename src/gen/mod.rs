use image::DynamicImage;

use crate::geo::Mesh;

/// Cylindrical lithophane generator
pub mod cylinder_mesh;
/// Image preprocessor with user-specified filter
pub mod filter_image;
/// Flat image lithophane generator
pub mod flat_mesh;
/// Standard image preprocessor
pub mod standard_image;

pub trait LithophaneGenerator {
    fn generate(self, source: LightMap) -> Mesh;
}

pub struct LightMap {
    lightnesses: Vec<f32>,
    dims: (usize, usize),
}

pub trait ImagePreprocessor {
    fn transform(self, image: &DynamicImage) -> LightMap;
}
