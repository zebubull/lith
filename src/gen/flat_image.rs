use image::{imageops::FilterType, DynamicImage};

use crate::{
    geo::{Mesh, Vec3},
    img::{luminance_to_lightness, srgb_to_luminance},
};

use super::LithophaneGenerator;

pub struct FlatImageGenerator {
    source: DynamicImage,
    scaling: f32,
    height: usize,
    width: usize,
    heights: Vec<f32>,
    bottom: f32,
    tris: Vec<Vec3>,
}

enum Side {
    Left,
    Top,
    Right,
    Bottom,
}

impl FlatImageGenerator {
    /// Set the target width of the output
    pub fn width(mut self, width: usize) -> Self {
        let new_height = self.source.height() * self.source.width() / width as u32;
        let img = self
            .source
            .resize(width as u32, new_height, FilterType::CatmullRom);
        self.width = img.width() as usize;
        self.height = img.height() as usize;
        self.source = img;
        self
    }

    /// Set the scale multiplier for the generator to use.
    pub fn scaling(mut self, scaling: f32) -> Self {
        // Negative scaling makes the lithophane work normally
        self.scaling = -scaling;
        self
    }

    /// Generate a heightmap for the current source and save it to `self.heights`
    fn generate_heightmap(&mut self) {
        self.heights.reserve(self.width * self.height);

        // Calculate the percieved lightness of each pixel and scale to get the final heightmap
        self.source
            .to_rgb8()
            .chunks_exact(3)
            .map(|p| srgb_to_luminance(p))
            .map(luminance_to_lightness)
            .map(|l| l / 100.0)
            .map(|l| l * self.scaling)
            .for_each(|h| {
                self.heights.push(h);
            });

        self.bottom = 1.0 * self.scaling;
    }

    /// Get the vertex at (x, y, heights[x, y])
    fn get_vertex(&self, x: usize, y: usize) -> Vec3 {
        Vec3 {
            x: x as f32,
            y: y as f32,
            z: self.heights[y * self.width + x],
        }
    }

    /// Get the vertex at (x, y, heights.min())
    fn get_bottom_vertex(&self, x: usize, y: usize) -> Vec3 {
        Vec3 {
            x: x as f32,
            y: y as f32,
            z: self.bottom,
        }
    }

    /// Add a quad whose bottom-right vertex is at (x, y)
    fn add_quad(&mut self, x: usize, y: usize) {
        let tl = self.get_vertex(x - 1, y - 1);
        let bl = self.get_vertex(x - 1, y);
        let tr = self.get_vertex(x, y - 1);
        let br = self.get_vertex(x, y);
        self.tris
            .extend_from_slice(&[br.clone(), bl, tl.clone(), tr, br, tl])
    }

    /// Add a quad on the brim of the image whose top-right vertex is at (x, y)
    fn add_brim_quad(&mut self, x: usize, y: usize, s: Side) {
        use Side::*;
        match s {
            Left => {
                let tl = self.get_vertex(x, y - 1);
                let tr = self.get_vertex(x, y);
                let bl = self.get_bottom_vertex(x, y - 1);
                let br = self.get_bottom_vertex(x, y);
                self.tris
                    .extend_from_slice(&[br.clone(), bl, tl.clone(), br, tl, tr])
            }
            Right => {
                let tl = self.get_vertex(x, y - 1);
                let tr = self.get_vertex(x, y);
                let bl = self.get_bottom_vertex(x, y - 1);
                let br = self.get_bottom_vertex(x, y);
                self.tris
                    .extend_from_slice(&[tl.clone(), bl, br.clone(), tr, tl, br])
            }
            Top => {
                let tl = self.get_vertex(x - 1, y);
                let tr = self.get_vertex(x, y);
                let bl = self.get_bottom_vertex(x - 1, y);
                let br = self.get_bottom_vertex(x, y);
                self.tris
                    .extend_from_slice(&[tl.clone(), bl, br.clone(), tr, tl, br])
            }
            Bottom => {
                let tl = self.get_vertex(x - 1, y);
                let tr = self.get_vertex(x, y);
                let bl = self.get_bottom_vertex(x - 1, y);
                let br = self.get_bottom_vertex(x, y);
                self.tris
                    .extend_from_slice(&[br.clone(), bl, tl.clone(), br, tl, tr])
            }
        }
    }

    fn add_bottom(&mut self) {
        let tl = self.get_bottom_vertex(0, 0);
        let tr = self.get_bottom_vertex(self.width - 1, 0);
        let bl = self.get_bottom_vertex(0, self.height - 1);
        let br = self.get_bottom_vertex(self.width - 1, self.height - 1);
        self.tris
            .extend_from_slice(&[br.clone(), bl, tl.clone(), br, tl, tr])
    }
}

impl From<DynamicImage> for FlatImageGenerator {
    fn from(value: DynamicImage) -> Self {
        Self {
            source: value,
            scaling: 0.0,
            width: 0,
            height: 0,
            heights: vec![],
            tris: vec![],
            bottom: f32::MAX,
        }
    }
}

impl LithophaneGenerator for FlatImageGenerator {
    fn generate(mut self) -> crate::geo::Mesh {
        assert_ne!(self.width, 0, "Width is 0. Did you forget to set it?");
        assert_ne!(self.height, 0, "Height is 0. Did you forget to set it?");

        self.generate_heightmap();
        for y in 1..self.height {
            for x in 1..self.width {
                self.add_quad(x, y);
            }

            self.add_brim_quad(0, y, Side::Left);
            self.add_brim_quad(self.width - 1, y, Side::Right);
        }

        for x in 1..self.width {
            self.add_brim_quad(x, 0, Side::Top);
            self.add_brim_quad(x, self.height - 1, Side::Bottom);
        }

        self.add_bottom();

        Mesh::new(self.tris)
    }
}
