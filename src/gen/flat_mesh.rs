use crate::geo::{Mesh, Vec3};

use super::{LightMap, LithophaneGenerator};

pub struct FlatMeshGenerator {
    scaling: f32,
    width: usize,
    heights: Vec<f32>,
    tris: Vec<Vec3>,
    bottom: f32,
}

enum Side {
    Left,
    Top,
    Right,
    Bottom,
}

impl FlatMeshGenerator {
    /// Set the scale multiplier for the generator to use.
    pub fn scaling(mut self, scaling: f32) -> Self {
        // Negative scaling makes the lithophane work normally
        self.scaling = -scaling;
        self
    }

    /// Generate a heightmap for the current source and save it to `self.heights`
    fn generate_heightmap(&mut self, source: LightMap) {
        self.heights.reserve(source.dims.0 * source.dims.1);

        // Calculate the percieved lightness of each pixel and scale to get the final heightmap
        source
            .lightnesses
            .iter()
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

    fn add_bottom(&mut self, width: usize, height: usize) {
        let tl = self.get_bottom_vertex(0, 0);
        let tr = self.get_bottom_vertex(width - 1, 0);
        let bl = self.get_bottom_vertex(0, height - 1);
        let br = self.get_bottom_vertex(width - 1, height - 1);
        self.tris
            .extend_from_slice(&[tl.clone(), bl, br.clone(), tr, tl, br])
    }
}

impl Default for FlatMeshGenerator {
    fn default() -> Self {
        Self {
            scaling: 1.0,
            width: 0,
            heights: vec![],
            tris: vec![],
            bottom: f32::MAX,
        }
    }
}

impl LithophaneGenerator for FlatMeshGenerator {
    fn generate(mut self, source: LightMap) -> crate::geo::Mesh {
        let (width, height) = source.dims;
        self.generate_heightmap(source);
        self.width = width;

        for y in 1..height {
            for x in 1..width {
                self.add_quad(x, y);
            }

            self.add_brim_quad(0, y, Side::Left);
            self.add_brim_quad(width - 1, y, Side::Right);
        }

        for x in 1..self.width {
            self.add_brim_quad(x, 0, Side::Top);
            self.add_brim_quad(x, height - 1, Side::Bottom);
        }

        self.add_bottom(width, height);

        Mesh::new(self.tris)
    }
}
