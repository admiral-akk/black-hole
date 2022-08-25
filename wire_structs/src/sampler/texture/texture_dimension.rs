use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct TextureDimension {
    pub size: usize,
}

impl TextureDimension {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
    pub fn v_01_to_index_weight(&self, v_01: f32) -> [(usize, f32); 2] {
        let v_01 = (self.size - 1) as f32 * v_01;
        let left_index = (v_01 as usize).clamp(0, self.size - 2);
        let right_index = left_index + 1;
        let right_weight = v_01 - left_index as f32;
        let left_weight = 1. - right_weight;
        [(left_index, left_weight), (right_index, right_weight)]
    }
}
