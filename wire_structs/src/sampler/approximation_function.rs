use std::f32::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use super::{
    gpu::simulated_ray::SimulatedRay, optimization_utils::optimize, simulated_path::SimulatedPath,
};

#[derive(Serialize, Deserialize)]
pub struct ApproximationFunction {
    theta_final: f32,
    min_distance: f32,
    start: f32,
}

trait MeasureError {
    fn error(&self, path: &SimulatedPath, angles: &Vec<f32>) -> f32;
}

pub trait Approximation {
    fn get_dist(&self, angle: f32) -> Option<f32>;
}

impl Approximation for ApproximationFunction {
    fn get_dist(&self, angle: f32) -> Option<f32> {
        if angle >= self.theta_final {
            return None;
        }
        let t = (self.theta_final / 2. - angle).abs().clamp(0., 1.);

        Some(
            (1. / (self.theta_final - angle) + 1. / self.theta_final + self.start) * t
                + (1. - t) * self.min_distance,
        )
    }
}

impl MeasureError for ApproximationFunction {
    fn error(&self, path: &SimulatedPath, angles: &Vec<f32>) -> f32 {
        let mut err = 0.;
        let mut count = 0;
        for (i, angle) in angles.iter().enumerate() {
            let dist = path.ray.angle_dist[i];
            if dist == 0. {
                count = i;
                break;
            }
            let dist = match self.get_dist(*angle) {
                Some(d) => d - dist,
                None => 10.,
            };
            err += dist * dist;
        }
        if count == 0 {
            return 0.;
        }
        return err / count as f32;
    }
}

impl ApproximationFunction {
    pub fn new(path: &SimulatedPath, angles: &Vec<f32>) -> Self {
        Self {
            theta_final: path.final_angle(angles),
            start: path.dist,
            min_distance: path.min_dist(),
        }
    }
}
