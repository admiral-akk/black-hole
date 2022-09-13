use std::f32::consts::{FRAC_PI_2, PI, TAU};

use serde::{Deserialize, Serialize};

use super::simulated_path::SimulatedPath;

#[derive(Serialize, Deserialize, Clone)]
pub struct ApproximationFunction {
    pub theta_start: f32,
    pub theta_final: f32,
    pub min_distance: f32,
    pub theta_max_start: f32,
    pub theta_min_start: f32,
    pub initial_dist: f32,
    pub view: f32,
}

trait MeasureError {
    fn error(&self, path: &SimulatedPath, angles: &Vec<f32>) -> f32;
}

pub trait Approximation {
    fn get_dist(&self, angle: f32) -> Option<f32>;
}

impl Approximation for ApproximationFunction {
    fn get_dist(&self, angle: f32) -> Option<f32> {
        if angle >= self.theta_final + FRAC_PI_2 || angle < 0. {
            return None;
        }

        if angle >= self.theta_final {
            return Some(self.min_distance / (angle - self.theta_final).cos());
        }

        if angle >= self.theta_min_start {
            return Some(self.min_distance);
        }

        return Some(self.min_distance / (self.theta_min_start - angle).cos());
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

fn optimal_min_dist(path: &SimulatedPath, angles: &Vec<f32>) -> f32 {
    let mut bounds = [0., 2.0];
    let theta_max_start = path.final_angle(angles) - FRAC_PI_2;
    if theta_max_start < FRAC_PI_2 {
        bounds[0] = path.dist * theta_max_start.cos();
    }

    while bounds[1] - bounds[0] > 0.001 {
        let min = bounds[0];
        let delta = bounds[1] - bounds[0];
        let left = min + delta / 3.;
        let right = min + 2. * delta / 3.;
        let left_f = ApproximationFunction::new(path, angles, left, 0.);
        let right_f = ApproximationFunction::new(path, angles, right, 0.);
        let left_err = left_f.error(path, angles);
        let right_err = right_f.error(path, angles);
        if left_err > right_err {
            bounds[0] = left;
        } else {
            bounds[1] = right;
        }
    }
    (bounds[1] + bounds[0]) / 2.
}

impl ApproximationFunction {
    pub fn generate(path: &SimulatedPath, angles: &Vec<f32>, view: f32) -> Self {
        let grazing_distance = path.grazing_distance();
        let min_distance;
        if path.ray.angle_dist[1] == 0. {
            min_distance = 0.001;
        } else if grazing_distance.is_some() {
            min_distance = grazing_distance.unwrap();
        } else {
            let final_angle = FRAC_PI_2 - path.final_angle_point(angles).min(FRAC_PI_2);
            min_distance = 1.0 * final_angle.cos();
        }
        ApproximationFunction::new(path, angles, min_distance, view)
    }

    fn new(path: &SimulatedPath, angles: &Vec<f32>, min_distance: f32, view: f32) -> Self {
        let theta_start = path.start_angle(angles);
        let theta_final = path.final_angle(angles);
        let theta_max_start = theta_final - FRAC_PI_2;
        let theta_min_start = (min_distance / path.dist).acos();
        let initial_dist = path.dist;
        Self {
            theta_start,
            theta_final,
            theta_max_start,
            theta_min_start,
            initial_dist,
            min_distance,
            view,
        }
    }
}
