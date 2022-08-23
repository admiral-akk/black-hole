use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize, Copy, Clone)]
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

    pub fn val_to_sample_params(&self, index_01: f32) -> [(usize, f32); 2] {
        let len = self.size;
        let (min, delta) = self.min_delta();
        let index_01 = (index_01 as f32 - min) / delta * (len - 1) as f32;
        let left_index = (index_01 as usize).clamp(0, len - 2);
        let right_index = left_index + 1;
        let right_weight = index_01 - left_index as f32;
        let left_weight = 1. - right_weight;
        [(left_index, left_weight), (right_index, right_weight)]
    }
}
