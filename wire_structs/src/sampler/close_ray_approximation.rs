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
    let mut points = Vec::new();
    for (i, &angle) in angle.generate_list().iter().enumerate() {
        if true_ray.angle_dist[i] == 0. {
            break;
        }
        let d = true_ray.angle_dist[i];
        points.push((d * angle.sin(), -d * angle.cos()));
    }
    let mut dense_points = Vec::new();
    for i in 1..points.len() {
        dense_points.push(points[i - 1]);
        let p_1 = points[i - 1];
        let p_2 = points[i];
        let d = ((p_2.0 - p_1.0).powi(2) + (p_2.1 - p_1.1).powi(2)).sqrt();
        for j in 1..(d as usize) {
            let t = j as f32 / d as f32;
            dense_points.push((t * p_1.0 + (1. - t) * p_2.0, t * p_1.1 + (1. - t) * p_2.1));
        }
    }
    if points.len() > 0 {
        dense_points.push(points[points.len() - 1]);

        for point in dense_points {
            let angle = f32::atan2(-point.1, point.0) + FRAC_PI_2;
            count += 1;
            let err = (point.0.powi(2) + point.1.powi(2)).sqrt() - approx.get_dist(angle);
            err_sq += err * err;
        }

        // for (i, &angle) in angle.generate_list().iter().enumerate() {
        //     if true_ray.angle_dist[i] == 0. {
        //         break;
        //     }
        //     count += 1;
        //     let err = true_ray.angle_dist[i] - approx.get_dist(angle);
        //     err_sq += err * err;
        // }
    }
    err_sq.sqrt() / count as f32
}

impl CloseRayApproximation {
    pub fn generate_optimal(ray: &SimulatedRay, start_dist: f32, angle: &DimensionParams) -> Self {
        let mut ray_copy = ray.clone();
        let mut increasing = false;
        for i in 1..ray_copy.angle_dist.len() {
            if ray_copy.angle_dist[i] > ray_copy.angle_dist[i - 1] {
                increasing = true;
            }
            if increasing {
                ray_copy.angle_dist[i] = 0.;
            }
        }
        let ray = &ray_copy;
        let final_angle_bounds = [0., TAU];
        let spiral_start_dist_bounds = [0., start_dist];

        let mut bounds = [final_angle_bounds, spiral_start_dist_bounds];
        let mut approx = optimize(
            &mut bounds,
            &move |params: &[f32]| CloseRayApproximation {
                start_dist,
                final_angle: TAU,
                spiral_start_angle: f32::acos(params[1] / start_dist),
                spiral_start_dist: params[1],
            },
            &move |approx: CloseRayApproximation| measure_error(&approx, ray, angle),
        );
        if approx.spiral_start_angle < 1. || approx.spiral_start_dist > 3. {
            approx.spiral_start_dist = 0.;
            approx.spiral_start_angle = FRAC_PI_2;
        }
        approx
    }

    pub fn get_dist(&self, angle: f32) -> f32 {
        if angle < self.spiral_start_angle {
            return self.spiral_start_dist / f32::cos(self.spiral_start_angle - angle);
        } else {
            let t =
                (angle - self.spiral_start_angle) / (self.final_angle - self.spiral_start_angle);
            return (1. - t) * self.spiral_start_dist;
        }
    }
}
