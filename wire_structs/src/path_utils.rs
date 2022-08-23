use std::f32::consts::TAU;

use crate::{
    angle_distance_cache::{
        generate_particle_with_fixed_dist, generate_particles, AngleDistanceCacheParams,
    },
    dimension_params::DimensionParams,
    gpu::gpu_state::simulate_particles,
};

const ITERATIONS: usize = 3;

pub fn find_closest_view(params: &AngleDistanceCacheParams) -> Vec<f32> {
    let dist = params.dist;
    let mut angle = params.angle;
    angle.bounds = [angle.bounds[0], 4. * TAU];
    angle.size = 64;
    let mut views: Vec<DimensionParams> = (0..dist.size)
        .map(|_| {
            let mut view = params.view_dist.clone();
            view.size = 32;
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
            .map(|(i, view)| generate_particle_with_fixed_dist(dists[i], &view, &params))
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
                    println!(
                        "ray_dist: {}, ray_index: {}, angle_index: {}",
                        dists[dist_index], ray_index, angle_index
                    );
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
        for (i, view) in views.iter().enumerate() {
            println!(
                "dist: {}, view_bounds: {:?}, max_angle: {}",
                dists[i], view.bounds, max_angles[i]
            )
        }
    }

    Vec::new()
}
