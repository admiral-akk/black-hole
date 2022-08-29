use serde::{Deserialize, Serialize};

use super::gpu::simulated_ray::SimulatedRay;

#[derive(Serialize, Deserialize)]
pub struct SimulatedPath {
    pub ray: SimulatedRay,
    pub dist: f32,
    pub view: f32,
}
