use std::f32::consts::TAU;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AngleDistanceCache {
    pub distances: Vec<Vec<Vec<f32>>>,
    pub params: AngleDistanceCacheParams,
}

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
    dimension_params::DimensionParams,
    gpu::{
        field::{Field, Particle},
        gpu_state::simulate_particles,
    },
    view_bounds_cache::ViewBoundsCache,
};

#[derive(Debug, PartialEq, Deserialize, Serialize, Copy, Clone)]
pub struct AngleDistanceCacheParams {
    pub dist: DimensionParams,
    pub view_dist: DimensionParams,
    pub angle: DimensionParams,
    pub black_hole_radius: f32,
    pub fov_degrees: f32,
}

impl AngleDistanceCacheParams {
    pub fn dimensions(&self) -> [u32; 3] {
        [
            self.angle.size as u32,
            self.view_dist.size as u32,
            self.dist.size as u32,
        ]
    }

    pub fn test_name(&self) -> String {
        format!(
            "dist_{:.1}_{:.1}_view_{:.4}_{:.4}_angle_{:.4}_{:.4}_radius_{:.1}",
            self.dist.bounds[0],
            self.dist.bounds[1],
            self.view_dist.bounds[0],
            self.view_dist.bounds[1],
            self.angle.bounds[0],
            self.angle.bounds[1],
            self.black_hole_radius,
        )
    }
    pub fn cache_name(&self) -> String {
        format!(
            "{}_{}_{}_dist_{:.1}_{:.1}_view_{:.4}_{:.4}_angle_{:.4}_{:.4}_radius_{:.1}",
            self.dist.size,
            self.view_dist.size,
            self.angle.size,
            self.dist.bounds[0],
            self.dist.bounds[1],
            self.view_dist.bounds[0],
            self.view_dist.bounds[1],
            self.angle.bounds[0],
            self.angle.bounds[1],
            self.black_hole_radius,
        )
    }

    pub fn to_vec2(&self, view_coord: f32, view_dist: &DimensionParams) -> Vec2 {
        let view_coord = map_view_coord(view_coord, view_dist);
        let view_width = (self.fov_degrees * TAU / 360.).tan();
        Vec2::new(view_width * view_coord, 1.).normalize()
    }

    fn index_01_to_sample_params(
        &self,
        dist: f32,
        view_coord: f32,
        angle: f32,
    ) -> ([(usize, f32); 2], [(usize, f32); 2], [(usize, f32); 2]) {
        let dists = self.dist.val_to_sample_params(dist);
        let zs = self
            .view_dist
            .val_to_sample_params(map_view_coord(view_coord, &self.view_dist));
        let angles = self.angle.val_to_sample_params(angle);
        (dists, zs, angles)
    }
}

fn map_view_coord(view_coord: f32, view_dist: &DimensionParams) -> f32 {
    let (min, delta) = &view_dist.min_delta();
    let view_01 = (view_coord - min) / delta;
    let view_01 = view_01 * view_01 * view_01 * view_01;
    view_01 * delta + min
}

impl AngleDistanceCache {
    pub fn get_dist(&self, dist: f64, view_port_coord: f64, angle: f64) -> Option<f32> {
        // check if z is in_bounds
        let view_z = view_port_coord as f32;
        if !self.params.view_dist.in_bounds(view_z) {
            return None;
        }
        let (dists, zs, angles) = self.params.index_01_to_sample_params(
            dist as f32,
            view_port_coord as f32,
            angle as f32,
        );
        let mut val = 0.;
        for (d, dw) in dists {
            for (z, zw) in zs {
                for (a, aw) in angles {
                    val += dw * zw * aw * self.distances[d][z][a];
                }
            }
        }
        Some(val)
    }
}

pub fn generate_particles(
    dist: &DimensionParams,
    view: &DimensionParams,
    params: &AngleDistanceCacheParams,
) -> Vec<Particle> {
    let dists = dist.generate_list();
    let views = view.generate_list();
    let mut particles = Vec::new();
    for d in 0..dists.len() {
        let dist = dists[d];
        let field = Field::new(1.5, dist as f64);
        for v in 0..views.len() {
            let view_coord = views[v];
            particles
                .push(field.spawn_particle(dist * Vec2::NEG_Y, params.to_vec2(view_coord, view)));
        }
    }
    particles
}

pub fn generate_particle_with_fixed_dist(
    dist: f32,
    view: &DimensionParams,
    params: &AngleDistanceCacheParams,
) -> Vec<Particle> {
    let views = view.generate_list();
    let mut particles = Vec::new();
    let field = Field::new(1.5, dist as f64);
    for v in 0..views.len() {
        let view_coord = views[v];
        particles.push(field.spawn_particle(dist * Vec2::NEG_Y, params.to_vec2(view_coord, view)));
    }
    particles
}

fn handle_zeros(angle_dist: &mut Vec<f32>, _angles: &Vec<f32>, _final_pos: [f32; 2]) {
    for i in 0..angle_dist.len() {
        if angle_dist[i] >= 1. {
            continue;
        }
        if i < 2 {
            continue;
        }
        if angle_dist[i - 1] > 5. {
            angle_dist[i] = 2. * angle_dist[i - 1] - angle_dist[i - 2];
        }
    }
}

impl AngleDistanceCache {
    pub fn generate_angle_distance_cache_gpu(
        params: &AngleDistanceCacheParams,
    ) -> AngleDistanceCache {
        let view_bounds =
            ViewBoundsCache::calculate_near_miss(&params.dist, &params.view_dist, &params.angle);

        let dists = params.dist.generate_list();
        let views_len = params.view_dist.size;
        let angles = params.angle.generate_list();

        let bound_views: Vec<DimensionParams> = dists
            .iter()
            .enumerate()
            .map(|(i, dist)| {
                let mut view = params.view_dist.clone();
                view.bounds[0] = view_bounds.get_bound(*dist as f64);
                view
            })
            .collect();

        let mut particles = Vec::new();
        for (i, view) in bound_views.iter().enumerate() {
            let view_particles = generate_particle_with_fixed_dist(dists[i], &view, &params);

            particles.extend_from_slice(&view_particles);
        }
        let rays = simulate_particles(particles, &params.angle, &params.dist);
        let mut distances: Vec<Vec<Vec<f32>>> = Vec::new();
        for d in 0..dists.len() {
            let mut fixed_distance = Vec::new();
            let paths = &rays[(d * views_len)..((d + 1) * views_len)];
            for v in 0..views_len {
                let ray = &paths[v];

                let mut angle_dist = ray.angle_dist.clone();
                //  handle_zeros(&mut angle_dist, &angles, ray.final_pos);
                fixed_distance.push(angle_dist);
            }
            distances.push(fixed_distance);
        }

        AngleDistanceCache {
            distances,
            params: params.clone(),
        }
    }
}

pub struct DualAngleDistanceCache {
    near_miss: ViewBoundsCache,
}

impl DualAngleDistanceCache {}
