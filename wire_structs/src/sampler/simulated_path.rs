use std::f32::consts::{FRAC_PI_2, PI, TAU};

use serde::{Deserialize, Serialize};

use super::gpu::simulated_ray::SimulatedRay;

#[derive(Serialize, Deserialize)]
pub struct SimulatedPath {
    pub ray: SimulatedRay,
    pub dist: f32,
    pub view: f32,
}

impl SimulatedPath {
    pub fn min_dist(&self) -> f32 {
        match self
            .ray
            .angle_dist
            .iter()
            .filter(|d| **d > 0.)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
        {
            Some(v) => *v,
            None => 1.5,
        }
    }

    pub fn start_angle(&self, angles: &Vec<f32>) -> f32 {
        let d_1 = self.ray.angle_dist[1];
        if d_1 == 0. {
            return 0.;
        }
        -f32::atan2(angles[1].sin(), -d_1 * angles[1].cos() + self.dist)
    }

    pub fn projected_min_angle(&self, angles: &Vec<f32>) -> f32 {
        (self.final_angle(&angles) + self.start_angle(&angles)) / 2.
    }

    pub fn final_angle(&self, angles: &Vec<f32>) -> f32 {
        let final_index = self
            .ray
            .angle_dist
            .iter()
            .enumerate()
            .filter(|(i, dist)| **dist > 0.)
            .last();
        let final_index = match final_index {
            Some((i, _)) => i,
            None => 0,
        };
        if self.ray.angle_dist[final_index] > 2. {
            return angles[final_index];
        } else {
            let final_dir = self.ray.final_dir;
            let final_angle = (f32::atan2(final_dir[1], final_dir[0]));
            let final_angle = (TAU - (final_angle - FRAC_PI_2));
            return final_angle;
        }
    }
}
