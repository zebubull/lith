use crate::geo::{Mesh, Vec3};

use super::{LightMap, LithophaneGenerator};

pub struct CylinderMeshGenerator {
    scaling: f32,
    width: usize,
    height: usize,
    size: f32,
    heights: Vec<f32>,
    tris: Vec<Vec3>,
    bottom: f32,
    radius: f32,
}

impl CylinderMeshGenerator {
    /// Set the scale multiplier for the generator to use.
    pub fn scaling(mut self, scaling: f32) -> Self {
        // Negative scaling makes the lithophane work normally
        self.scaling = -scaling;
        self
    }
    
    /// Set the target height of the cylinder
    pub fn height(mut self, height: f32) -> Self {
        // the field `height` is used for the height of the source image
        self.size = height;
        self
    }
    
    /// Set the target radius of the cylinder
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
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
        let angle = (x as f32 / self.width as f32) * 2.0 * std::f32::consts::PI;
        let (sin, cos) = angle.sin_cos();
        let height = self.heights[y * self.width + x];
        let radius = self.radius + height;
        Vec3 {
            x: radius * cos,
            y: radius * sin,
            z: -(y as f32 / self.height as f32) * self.size,
        }
    }

    fn get_interior_vertex(&self, x: usize, y: usize) -> Vec3 {
        let angle = (x as f32 / self.width as f32) * 2.0 * std::f32::consts::PI;
        let (sin, cos) = angle.sin_cos();
        let radius = self.radius + self.bottom;
        Vec3 {
            x: radius * cos,
            y: radius * sin,
            z: -(y as f32 / self.height as f32) * self.size,
        }
    }


    /// Add a quad whose bottom-right vertex is at (x, y)
    fn add_quad(&mut self, x: usize, y: usize) {
        let tl = self.get_vertex(x - 1, y - 1);
        let bl = self.get_vertex(x - 1, y);
        let tr = self.get_vertex(x, y - 1);
        let br = self.get_vertex(x, y);
        self.tris
            .extend_from_slice(&[tl.clone(), bl, br.clone(), tl, br, tr])
    }

    fn add_interior_quad(&mut self, x: usize, y: usize) {
        let tl = self.get_interior_vertex(x - 1, y - 1);
        let bl = self.get_interior_vertex(x - 1, y);
        let tr = self.get_interior_vertex(x, y - 1);
        let br = self.get_interior_vertex(x, y);
        self.tris
            .extend_from_slice(&[br.clone(), bl, tl.clone(), tr, br, tl])
    }

    fn bridge_edge_loop(&mut self, y: usize) {
            let tl = self.get_vertex(self.width - 1, y - 1);
            let bl = self.get_vertex(self.width - 1, y);
            let tr = self.get_vertex(0, y - 1);
            let br = self.get_vertex(0, y);
            self.tris
                .extend_from_slice(&[tl.clone(), bl, br.clone(), tl, br, tr]);
            let tl = self.get_interior_vertex(self.width -1, y - 1);
            let bl = self.get_interior_vertex(self.width - 1, y);
            let tr = self.get_interior_vertex(0, y - 1);
            let br = self.get_interior_vertex(0, y);
            self.tris
                .extend_from_slice(&[br.clone(), bl, tl.clone(), tr, br, tl]);
    }

    fn bridge_int_ext(&mut self, x: usize) {
        let tl = self.get_interior_vertex(x - 1, self.height - 1);
        let bl = self.get_vertex(x - 1, self.height - 1);
        let tr = self.get_interior_vertex(x, self.height - 1);
        let br = self.get_vertex(x, self.height - 1);
        self.tris
            .extend_from_slice(&[br.clone(), bl, tl.clone(), tr, br, tl]);
        
        let tl = self.get_vertex(x - 1, 0);
        let bl = self.get_interior_vertex(x - 1, 0);
        let tr = self.get_vertex(x, 0);
        let br = self.get_interior_vertex(x, 0);
        self.tris
            .extend_from_slice(&[br.clone(), bl, tl.clone(), tr, br, tl]);
    }

    fn bridge_int_ext_loop(&mut self) {
        let tl = self.get_interior_vertex(self.width - 1, self.height - 1);
        let bl = self.get_vertex(self.width - 1, self.height - 1);
        let tr = self.get_interior_vertex(0, self.height - 1);
        let br = self.get_vertex(0, self.height - 1);
        self.tris
            .extend_from_slice(&[br.clone(), bl, tl.clone(), tr, br, tl]);
        
        let tl = self.get_vertex(self.width - 1, 0);
        let bl = self.get_interior_vertex(self.width - 1, 0);
        let tr = self.get_vertex(0, 0);
        let br = self.get_interior_vertex(0, 0);
        self.tris
            .extend_from_slice(&[br.clone(), bl, tl.clone(), tr, br, tl]);
    }
}

impl Default for CylinderMeshGenerator {
    fn default() -> Self {
        Self {
            scaling: 1.0,
            width: 0,
            height: 0,
            radius: 0.0,
            size: 0.0,
            heights: vec![],
            tris: vec![],
            bottom: f32::MAX,
        }
    }
}

impl LithophaneGenerator for CylinderMeshGenerator {
    fn generate(mut self, source: LightMap) -> crate::geo::Mesh {
        let (width, height) = source.dims;
        self.generate_heightmap(source);
        self.width = width;
        self.height = height;

        for y in 1..height {
            for x in 1..width {
                self.add_quad(x, y);
                self.add_interior_quad(x, y);
            }
    
            self.bridge_edge_loop(y);
        }

        for x in 1..width {
            self.bridge_int_ext(x);
        }

        self.bridge_int_ext_loop();

        Mesh::new(self.tris)
    }
}
