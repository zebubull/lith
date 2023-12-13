use crate::img::{luminance_to_lightness, srgb_to_luminance};

use super::{ImagePreprocessor, LightMap};
use image::imageops::FilterType;

pub struct FilterImagePreprocessor {
    width: usize,
    filter: FilterType,
}

impl FilterImagePreprocessor {
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn filter(mut self, filter: FilterType) -> Self {
        self.filter = filter;
        self
    }
}

impl Default for FilterImagePreprocessor {
    fn default() -> Self {
        Self {
            width: 0,
            filter: FilterType::CatmullRom,
        }
    }
}

impl ImagePreprocessor for FilterImagePreprocessor {
    fn transform(self, image: &image::DynamicImage) -> LightMap {
        let image = image.resize(self.width as u32, image.height(), self.filter);
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
