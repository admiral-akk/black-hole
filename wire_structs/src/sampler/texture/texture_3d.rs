use serde::{Deserialize, Serialize};

use super::texture_dimension::TextureDimension;

#[derive(Serialize, Deserialize, Clone)]
pub struct Texture3D {
    dimensions: [TextureDimension; 3],
    val: Vec<f32>,
}

impl Texture3D {
    pub fn new(dimensions: [usize; 3]) -> Self {
        Self {
            dimensions: [
                TextureDimension::new(dimensions[0]),
                TextureDimension::new(dimensions[1]),
                TextureDimension::new(dimensions[2]),
            ],
            val: vec![0.; dimensions[0] * dimensions[1] * dimensions[2]],
        }
    }

    pub fn insert(&mut self, indices: [usize; 3], val: f32) {
        self.val[indices[2]
            + (indices[1] + indices[0] * self.dimensions[1].size) * self.dimensions[2].size] = val;
    }

    pub fn get(&self, indices_01: [f32; 3]) -> f32 {
        let mut val = 0.;
        for (x, x_w) in self.dimensions[0].v_01_to_index_weight(indices_01[0]) {
            for (y, y_w) in self.dimensions[1].v_01_to_index_weight(indices_01[1]) {
                for (z, z_w) in self.dimensions[2].v_01_to_index_weight(indices_01[2]) {
                    let index = z + (y + x * self.dimensions[1].size) * self.dimensions[2].size;
                    val += x_w * y_w * z_w * self.val[index];
                }
            }
        }
        val
    }
}
