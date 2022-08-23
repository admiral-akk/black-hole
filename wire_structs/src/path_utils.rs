use std::f32::consts::TAU;

use crate::{
    angle_distance_cache::{
        generate_particle_with_fixed_dist, generate_particles, AngleDistanceCacheParams,
    },
    dimension_params::DimensionParams,
    gpu::gpu_state::simulate_particles,
};

const ITERATIONS: usize = 2;

pub fn find_closest_view(params: &AngleDistanceCacheParams) -> Vec<f32> {
    let dist = params.dist;
    let view = params.view_dist;
    let mut angle = params.angle;
    angle.bounds = [angle.bounds[0], 4. * TAU];
    angle.size *= 8;
    let mut views: Vec<DimensionParams> = (0..dist.size).map(|_| view.clone()).collect();

    let dists = dist.generate_list();
    let angles = angle.generate_list();

    let mut max_angles: Vec<f32> = (0..views.len()).map(|i| 0.).collect();

    for _ in 0..ITERATIONS {
        let particles = views
            .iter()
            .enumerate()
            .map(|(i, view)| generate_particle_with_fixed_dist(dists[i], &view, &params))
            .fold(Vec::new(), |mut acc, p| {
                acc.extend_from_slice(&p);
                acc
            });
        let rays = simulate_particles(particles, &angle, &dist);
        for dist_index in 0..dist.size {
            let rays = &rays[(dist_index * view.size)..(dist_index + 1) * view.size];
            let mut longest_ray_index = 0;
            let mut max_len = 0;

            for (i, ray) in rays.iter().enumerate() {
                let zero_index = ray
                    .angle_dist
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| **v == 0.)
                    .map(|(index, _)| index)
                    .find(|_| true)
                    .unwrap_or_default();
                if max_len < zero_index {
                    longest_ray_index = i;
                    max_len = zero_index;
                    max_angles[dist_index] = angles[i];
                }
            }

            // the longest possible ray lies between (longest_ray_index-1),(longest_ray_index)
            let view = &views[dist_index];
            let indices = view.generate_list();
            let (min, max) = (indices[longest_ray_index - 1], indices[longest_ray_index]);
            views[dist_index].bounds = [min, max];
        }
    }

    for (i, view) in views.iter().enumerate() {
        println!(
            "dist: {}, view_bounds: {:?}, max_angle: {}",
            dists[i], view.bounds, max_angles[i]
        )
    }

    Vec::new()
}
