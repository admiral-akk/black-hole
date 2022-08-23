use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DimensionParams {
    pub size: usize,
    pub bounds: [f32; 2],
}

impl DimensionParams {
    pub fn min_delta(&self) -> (f32, f32) {
        (self.bounds[0], self.bounds[1] - self.bounds[0])
    }

    pub fn generate_list(&self) -> Vec<f32> {
        let (min, delta) = self.min_delta();
        (0..self.size)
            .map(|i| i as f32 / (self.size - 1) as f32)
            .map(|i_01| min + delta * i_01)
            .collect()
    }

    pub fn in_bounds(&self, val: f32) -> bool {
        self.bounds[0] <= val && val <= self.bounds[1]
    }
}
