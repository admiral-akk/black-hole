use std::{
    f32::consts::{PI, TAU},
    ops::Mul,
};

use serde::{Deserialize, Serialize};

use super::{
    dimension_params::DimensionParams, gpu::simulated_ray::SimulatedRay,
    optimization_utils::optimize,
};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct CloseRayApproximation {
    pub start_dist: f32,
    pub final_angle: f32,
    pub spiral_start_angle: f32,
    pub spiral_start_dist: f32,
}

pub fn measure_error(
    approx: &CloseRayApproximation,
    true_ray: &SimulatedRay,
    angle: &DimensionParams,
) -> f32 {
    let mut err_sq = 0.;
    let mut count = 0;
    for (i, &angle) in angle.generate_list().iter().enumerate() {
        if true_ray.angle_dist[i] == 0. {
            break;
        }
        count += 1;
        let err = true_ray.angle_dist[i] - approx.get_dist(angle);
        err_sq += err * err;
    }
    err_sq.sqrt() / count as f32
}

impl CloseRayApproximation {
    pub fn generate_optimal(ray: &SimulatedRay, start_dist: f32, angle: &DimensionParams) -> Self {
        let final_angle_bounds = [0., TAU];
        let spiral_start_dist_bounds = [0., start_dist];

        let mut bounds = [final_angle_bounds, spiral_start_dist_bounds];
        optimize(
            &mut bounds,
            &move |params: &[f32]| CloseRayApproximation {
                start_dist,
                final_angle: params[0],
                spiral_start_angle: f32::acos(params[1] / start_dist),
                spiral_start_dist: params[1],
            },
            &move |approx: CloseRayApproximation| measure_error(&approx, ray, angle),
        )
    }

    pub fn get_dist(&self, angle: f32) -> f32 {
        if angle < self.spiral_start_angle {
            return self.spiral_start_dist / f32::cos(self.spiral_start_angle - angle);
        } else {
            let t =
                (angle - self.spiral_start_angle) / (self.final_angle - self.spiral_start_angle);
            return (1. - t) * self.spiral_start_dist + t * 1.5;
        }
    }
}
