use std::f32::consts::TAU;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ViewBoundsCache {
    pub view_thresholds: Vec<f32>,
    pub dist: DimensionParams,
    pub view: DimensionParams,
}

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::gpu::field::{Field, Particle};
use crate::{dimension_params::DimensionParams, gpu::gpu_state::simulate_particles};

const ITERATIONS: usize = 3;

fn find_closest_view(
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
) -> Vec<f32> {
    let angle = DimensionParams {
        bounds: [angle.bounds[0], TAU],
        size: 128,
    };
    let mut views: Vec<DimensionParams> = (0..dist.size)
        .map(|_| {
            let mut view = view.clone();
            view.size = 64;
            view
        })
        .collect();

    let dists = dist.generate_list();
    let angles = angle.generate_list();

    let mut max_angles: Vec<f32> = (0..views.len()).map(|_| 0.).collect();

    for _ in 0..ITERATIONS {
        let particles = views
            .iter()
            .enumerate()
            .map(|(i, view)| generate_particle_with_fixed_dist(dists[i], &view))
            .fold(Vec::new(), |mut acc, p| {
                acc.extend_from_slice(&p);
                acc
            });
        let rays = simulate_particles(particles, &angle, &dist);
        for dist_index in 0..dist.size {
            let rays = &rays
                [(dist_index * views[dist_index].size)..(dist_index + 1) * views[dist_index].size];
            let mut longest_ray_index = 0;
            let mut max_len = 0;
            for (ray_index, ray) in rays.iter().enumerate() {
                let mut angle_index = 0;
                for (j, dist) in ray.angle_dist.iter().enumerate() {
                    angle_index = j;
                    if *dist <= 0. {
                        break;
                    }
                }
                if max_len < angle_index {
                    longest_ray_index = ray_index;
                    max_len = angle_index;
                    max_angles[dist_index] = angles[angle_index];
                }
            }

            // the longest possible ray lies between (longest_ray_index-1),(longest_ray_index)
            let view = &views[dist_index];
            let indices = view.generate_list();

            let (min, max) = (
                indices[match longest_ray_index {
                    0 => 0,
                    v => v - 1,
                }],
                indices[longest_ray_index],
            );
            views[dist_index].bounds = [min, max];
        }
    }

    let mut final_views = Vec::new();

    for view in views.iter() {
        final_views.push(view.bounds[1]);
    }

    final_views
}

impl ViewBoundsCache {
    pub fn get_bound(&self, dist: f64) -> f32 {
        // check if z is in_bounds
        let dists = self.dist.val_to_sample_params(dist as f32);
        let mut val = 0.;
        for (d, dw) in dists {
            val += dw * self.view_thresholds[d];
        }
        val
    }
}

fn generate_particle_with_fixed_dist(dist: f32, view: &DimensionParams) -> Vec<Particle> {
    let views = view.generate_list();
    let mut particles = Vec::new();
    let field = Field::new(1.5, dist as f64);
    for v in 0..views.len() {
        let view = views[v];
        let view_width = (60. * TAU / 360.).tan();
        let coord = Vec2::new(view_width * view, 1.).normalize();
        particles.push(field.spawn_particle(dist * Vec2::NEG_Y, coord));
    }
    particles
}

impl ViewBoundsCache {
    pub fn calculate_near_miss(
        dist: &DimensionParams,
        view: &DimensionParams,
        angle: &DimensionParams,
    ) -> ViewBoundsCache {
        ViewBoundsCache {
            view_thresholds: find_closest_view(dist, view, angle),
            dist: dist.clone(),
            view: view.clone(),
        }
    }
}
