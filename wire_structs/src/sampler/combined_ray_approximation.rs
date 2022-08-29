use std::{
    f32::consts::{FRAC_PI_2, PI, TAU},
    ops::Mul,
};

use serde::{Deserialize, Serialize};

use super::{
    dimension_params::DimensionParams, gpu::simulated_ray::SimulatedRay,
    optimization_utils::optimize,
};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct CombinedRayApproximation {
    pub curve_dist: f32,
    pub final_angle: f32,
    pub curve_start_angle: f32,
    pub close_weight: f32,
}

const FINAL_ANGLE_OFFSET: f32 = 0.05;

pub fn measure_error(
    approx: &CombinedRayApproximation,
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
        let err = true_ray.angle_dist[i] * (true_ray.angle_dist[i] - approx.get_dist(angle));
        err_sq += err * err;
    }
    err_sq.sqrt() / count as f32
}

impl CombinedRayApproximation {
    pub fn generate_optimal(ray: &SimulatedRay, start_dist: f32, angle: &DimensionParams) -> Self {
        let mut ray_copy = ray.clone();
        let ray = &ray_copy;
        let spiral_start_dist_bounds = [0., start_dist];

        let close_weight_bounds = [0., 1.];
        let mut bounds = [spiral_start_dist_bounds, close_weight_bounds];

        let final_angle = ray.final_angle();

        let mut approx = optimize(
            &mut bounds,
            &move |params: &[f32]| CombinedRayApproximation {
                curve_dist: params[0],
                final_angle: final_angle,
                curve_start_angle: f32::acos(params[0] / start_dist),
                close_weight: params[1],
            },
            &move |approx: CombinedRayApproximation| measure_error(&approx, ray, angle),
        );
        approx
    }

    pub fn get_dist(&self, angle: f32) -> f32 {
        if angle < self.curve_start_angle {
            return self.curve_dist / f32::cos(self.curve_start_angle - angle);
        } else {
            let close_t = ((self.final_angle - angle)
                / (self.final_angle - self.curve_start_angle))
                .clamp(0., 1.);
            let close = close_t * self.curve_dist + (1. - close_t) * 0.;

            let far;
            let fake_final_angle = self.final_angle + FINAL_ANGLE_OFFSET;
            if angle > fake_final_angle - FRAC_PI_2 {
                if angle > fake_final_angle {
                    far = 0.;
                } else {
                    far = self.curve_dist / f32::cos(angle - (fake_final_angle - FRAC_PI_2));
                }
            } else {
                far = self.curve_dist;
            }
            return self.close_weight * close + (1. - self.close_weight) * far;
        }
    }
}
