use super::texture_dimension::TextureDimension;

pub struct Texture1D {
    dimensions: [TextureDimension; 1],
    val: Vec<f32>,
}

impl Texture1D {
    pub fn new(dimensions: [usize; 1]) -> Self {
        Self {
            dimensions: [TextureDimension::new(dimensions[0])],
            val: vec![0.; dimensions[0]],
        }
    }

    pub fn insert(&mut self, indices: [usize; 1], val: f32) {
        self.val[indices[0]] = val;
    }

    pub fn get(&self, indices_01: [f32; 1]) -> f32 {
        let mut val = 0.;
        for (x, x_w) in self.dimensions[0].v_01_to_index_weight(indices_01[0]) {
            val += x_w * self.val[x];
        }
        val
    }
}
