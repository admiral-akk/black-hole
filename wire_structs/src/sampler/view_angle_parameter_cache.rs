use serde::{Deserialize, Serialize};

use super::approximation_function::ApproximationFunction;

#[derive(Serialize, Deserialize)]
pub struct ViewAngleParameterCache {
    pub dist_dim: u32,
    pub params: Vec<ApproximationFunction>,
}
impl ViewAngleParameterCache {
    pub fn new(dist_dim: u32, params: &Vec<ApproximationFunction>) -> Self {
        let mut params_copy = Vec::new();
        params_copy.clone_from(params);
        ViewAngleParameterCache {
            dist_dim,
            params: params_copy,
        }
    }
}
