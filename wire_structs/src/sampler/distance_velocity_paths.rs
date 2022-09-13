use std::f32::consts::{PI, TAU};

use glam::Vec2;

use super::{
    dimension_params::DimensionParams,
    gpu::{field::Particle, gpu_state::simulate_particles},
    simulated_path::SimulatedPath,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DistanceVelocityPaths {
    pub paths: Vec<SimulatedPath>,
    pub velocity_bounds: (f32, f32),
}
const SAMPLES_PER_DIST: usize = 1 << 6;
const ITERATIONS: u32 = 5;

impl DistanceVelocityPaths {
    pub fn velocity_at(&self, dist: f32) -> f32 {
        let start_pe = 0.25 * (dist.powi(-4) - 1.5_f32.powi(-4));
        let final_v = (self.velocity_bounds.1 + self.velocity_bounds.0) / 2.;
        let final_ke = 0.5 * final_v * final_v;
        let start_ke = final_ke + start_pe;
        (2. * start_ke).sqrt()
    }

    pub fn new() -> Self {
        let angles = DimensionParams {
            size: 360,
            bounds: [0., TAU],
        };

        let mut velocity_bounds = (0., 2.5);

        let mut ret = Vec::new();
        for _ in 0..ITERATIONS {
            let mut test = Vec::new();
            let initial_dir = Vec2::new(1., 0.);
            for sample_index in 0..SAMPLES_PER_DIST {
                let velocity = sample_index as f32 / (SAMPLES_PER_DIST - 1) as f32;
                let velocity =
                    velocity * (velocity_bounds.1 - velocity_bounds.0) + velocity_bounds.0;
                let particle = Particle::new(1.5, initial_dir, velocity);
                test.push(particle);
            }
            let paths = simulate_particles(test, &angles, 40.);

            ret = Vec::new();
            for ray in &paths {
                let mut ray = ray.clone();
                ray.angle_dist = ray
                    .angle_dist
                    .iter()
                    .map(|v| match f32::is_nan(*v) {
                        true => 0.,
                        false => *v,
                    })
                    .collect();
                let new_path = SimulatedPath {
                    ray: ray.clone(),
                    dist: 1.5,
                    view: 0.,
                };
                ret.push(new_path);
            }
            let mut opt = 0;
            let mut opt_dist = 100.;
            let mut max_index = 0;
            for (r_i, ray) in paths.iter().enumerate() {
                let final_pos = Vec2::from_array(ray.final_pos);
                let err = (final_pos.length() - 1.5).abs() as f64;
                let mut furtherest_index = 0;
                for (i, d) in ray.angle_dist.iter().enumerate() {
                    if *d <= 0. {
                        break;
                    }
                    furtherest_index = i;
                }
                if furtherest_index > max_index || (opt_dist > err && furtherest_index == max_index)
                {
                    opt_dist = err;
                    opt = r_i;
                    max_index = furtherest_index;
                }
            }
            let min = match opt == 0 {
                true => opt,
                false => opt - 1,
            };
            let min_vel = min as f32 / (SAMPLES_PER_DIST - 1) as f32;
            let min_vel = min_vel * (velocity_bounds.1 - velocity_bounds.0) + velocity_bounds.0;
            let max = match opt == paths.len() - 1 {
                true => opt,
                false => opt + 1,
            };
            let max_vel = max as f32 / (SAMPLES_PER_DIST - 1) as f32;
            let max_vel = max_vel * (velocity_bounds.1 - velocity_bounds.0) + velocity_bounds.0;
            velocity_bounds = (min_vel, max_vel);
        }

        Self {
            paths: ret,
            velocity_bounds,
        }
    }
}
