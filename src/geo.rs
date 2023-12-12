#[derive(Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn to_bytes(&self) -> [u8; 12] {
        let x = self.x.to_le_bytes();
        let y = self.y.to_le_bytes();
        let z = self.z.to_le_bytes();
        [
            x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3], z[0], z[1], z[2], z[3],
        ]
    }
}

impl std::ops::Sub<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

trait CalcNormal {
    fn normal(&self) -> Vec3;
}

impl CalcNormal for [Vec3; 3] {
    fn normal(&self) -> Vec3 {
        let u = &self[1] - &self[0];
        let v = &self[2] - &self[0];
        Vec3 {
            x: u.y * v.z - u.z * v.y,
            y: u.z * v.x - u.x * v.z,
            z: u.x * v.y - u.y * v.x,
        }
    }
}

pub struct Mesh {
    vertices: Vec<Vec3>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vec3>) -> Self {
        Self { vertices }
    }
    pub fn as_stl_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&[0; 80]);
        bytes.extend_from_slice(&((self.vertices.len() / 3) as u32).to_le_bytes());

        self.vertices.chunks_exact(3).for_each(|t| {
            let t: &[Vec3; 3] = t.try_into().unwrap();
            bytes.extend_from_slice(&t.normal().to_bytes());
            t.iter()
                .for_each(|v| bytes.extend_from_slice(&v.to_bytes()));
            bytes.extend_from_slice(&[0, 0])
        });

        bytes
    }
}
