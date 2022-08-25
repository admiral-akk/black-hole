use crate::sampler::ray_approximation::RayApproximation;

use super::texture_dimension::TextureDimension;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Texture2D {
    dimensions: [TextureDimension; 2],
    val: Vec<RayApproximation>,
}

impl Texture2D {
    pub fn new(dimensions: [usize; 2]) -> Self {
        Self {
            dimensions: [
                TextureDimension::new(dimensions[0]),
                TextureDimension::new(dimensions[1]),
            ],
            val: vec![RayApproximation::default(); dimensions[0] * dimensions[1]],
        }
    }

    pub fn insert(&mut self, indices: [usize; 2], approx: RayApproximation) {
        self.val[indices[1] + indices[0] * self.dimensions[1].size] = approx;
    }

    pub fn get(&self, indices_01: [f32; 2]) -> RayApproximation {
        let mut approx = Vec::new();
        for (x, x_w) in self.dimensions[0].v_01_to_index_weight(indices_01[0]) {
            for (y, y_w) in self.dimensions[1].v_01_to_index_weight(indices_01[1]) {
                let index = y + x * self.dimensions[1].size;
                approx.push(x_w * y_w * self.val[index]);
            }
        }
        4. * RayApproximation::generate_average(&approx)
    }
}
#[cfg(test)]
mod tests {
    use crate::sampler::ray_approximation::RayApproximation;

    use super::Texture2D;

    #[test]
    fn linear() {
        let mut tex = Texture2D::new([128, 256]);
        for i in 0..128 {
            for j in 0..256 {
                let (i_01, j_01) = (i as f32 / 127., j as f32 / 255.);
                let v = RayApproximation::new(i_01, j_01, i_01 + j_01);
                tex.insert([i, j], v);
            }
        }

        for x in 0..10000 {
            for y in 0..10000 {
                let x_01 = x as f32 / 9999.;
                let y_01 = y as f32 / 9999.;
                let true_v = RayApproximation::new(x_01, y_01, x_01 + y_01);
                let approx_v = tex.get([x_01, y_01]);
                assert!(
                    (approx_v.curve_dist - true_v.curve_dist).abs()
                        + (approx_v.final_angle - true_v.final_angle).abs()
                        + (approx_v.start_dist - true_v.start_dist).abs()
                        < 0.0001,
                    "\nLinear approximation function not equal.\nx_01: {}\ny_01: {}\ntrue: {:?}\napprox: {:?}",
                    x_01,
                    y_01,
                    true_v,
                    approx_v
                );
            }
        }
    }
}
