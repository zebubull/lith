use crate::img::{luminance_to_lightness, srgb_to_luminance};

use super::{ImagePreprocessor, LightMap};

#[derive(Default)]
pub struct StandardImagePreprocessor {
    width: usize,
}

impl StandardImagePreprocessor {
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }
}

impl ImagePreprocessor for StandardImagePreprocessor {
    fn transform(self, image: &image::DynamicImage) -> LightMap {
        let image = image.resize(
            self.width as u32,
            image.height(),
            image::imageops::FilterType::CatmullRom,
        );
        let lights: Vec<_> = image
            .to_rgb8()
            .chunks_exact(3)
            .map(srgb_to_luminance)
            .map(luminance_to_lightness)
            .map(|l| l / 100.0)
            .collect();
        LightMap {
            lightnesses: lights,
            dims: (image.width() as usize, image.height() as usize),
        }
    }
}
