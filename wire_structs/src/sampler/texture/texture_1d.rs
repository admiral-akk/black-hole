use super::texture_dimension::TextureDimension;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
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
#[cfg(test)]
mod tests {
    use super::Texture1D;

    #[test]
    fn linear() {
        let mut tex = Texture1D::new([128]);
        for i in 0..128 {
            let v = i as f32 / 127.;
            tex.insert([i], v);
        }

        for x in 0..10000 {
            let x_01 = x as f32 / 9999.;
            let true_v = x as f32 / 9999.;
            let approx_v = tex.get([x_01]);
            assert!(
                (true_v - approx_v).abs() < 0.00001,
                "\nLinear function not equal.\nx_01: {}\ntrue: {}\napprox: {}",
                x_01,
                true_v,
                approx_v
            );
        }
    }
}
