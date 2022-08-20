use std::f32::consts::TAU;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AngleDistanceCache {
    pub distances: Vec<Vec<Vec<f32>>>,
    pub params: AngleDistanceCacheParams,
}

use serde::{Deserialize, Serialize};

use crate::path_integration::path::cast_ray_steps_response;
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DimensionParams {
    pub size: usize,
    pub bounds: [f32; 2],
}

impl DimensionParams {
    pub fn min_delta(&self) -> (f32, f32) {
        (self.bounds[0], self.bounds[1] - self.bounds[0])
    }

    pub fn generate_list(&self) -> Vec<f32> {
        let (min, delta) = self.min_delta();
        (0..self.size)
            .map(|i| i as f32 / (self.size - 1) as f32)
            .map(|i_01| min + delta * i_01)
            .collect()
    }

    fn in_bounds(&self, val: f32) -> bool {
        self.bounds[0] <= val && val <= self.bounds[1]
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AngleDistanceCacheParams {
    pub dist: DimensionParams,
    pub view_dist: DimensionParams,
    pub angle: DimensionParams,
    pub black_hole_radius: f32,
    pub fov_degrees: f32,
}

impl AngleDistanceCacheParams {
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

    pub fn to_z(&self, view_coord: f32) -> f32 {
        let view_width = 2. * (self.fov_degrees * TAU / 360.).tan();
        1. / (1. + view_width * 2.0_f32.sqrt() * view_coord).sqrt()
    }

    fn to_view(&self, z: f32) -> f32 {
        let view_width = 2. * (self.fov_degrees * TAU / 360.).tan();
        (1. / (z * z) - 1.) / (view_width * 2.0_f32.sqrt())
    }
}

fn index_01_to_sample_params(index_01: f64, dimension: &DimensionParams) -> [(usize, f32); 2] {
    let len = dimension.size;
    let (min, delta) = dimension.min_delta();
    let index_01 = (index_01 as f32 - min) / delta * (len - 1) as f32;
    let left_index = (index_01 as usize).clamp(0, len - 2);
    let right_index = left_index + 1;
    let right_weight = index_01 - left_index as f32;
    let left_weight = 1. - right_weight;
    [(left_index, left_weight), (right_index, right_weight)]
}

impl AngleDistanceCache {
    pub fn get_dist(&self, dist: f64, view_port_coord: f64, angle: f64) -> Option<f32> {
        // check if z is in_bounds
        let view_z = view_port_coord as f32;
        if !self.params.view_dist.in_bounds(view_z) {
            return None;
        }
        let dists = index_01_to_sample_params(dist, &self.params.dist);
        let zs = index_01_to_sample_params(view_z as f64, &self.params.view_dist);
        let angles = index_01_to_sample_params(angle, &self.params.angle);
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

impl AngleDistanceCache {
    pub fn generate_angle_distance_cache(params: AngleDistanceCacheParams) -> AngleDistanceCache {
        let dists = params.dist.generate_list();
        let views = params.view_dist.generate_list();
        let angles = params.angle.generate_list();

        let mut distances: Vec<Vec<Vec<f32>>> = Vec::new();
        for d in 0..dists.len() {
            let mut camera_dist: Vec<Vec<f32>> = Vec::new();
            let dist = dists[d];
            for view in 0..views.len() {
                let mut view_dist: Vec<f32> = Vec::new();
                let view_coord = views[view];
                let z = params.to_z(view_coord);
                let response =
                    cast_ray_steps_response(z as f64, dist as f64, params.black_hole_radius as f64);
                for a in 0..angles.len() {
                    let angle = angles[a];

                    let distance = match response.get_angle_dist().get_dist(angle as f64) {
                        Some(distance) => distance as f32,
                        None => match response.hits_black_hole() {
                            true => 0.,
                            false => params.dist.bounds[1],
                        },
                    };

                    println!(
                        "Distance: ({}/{}, {}/{}, {}/{}): {}",
                        d,
                        dists.len(),
                        view,
                        views.len(),
                        a,
                        angles.len(),
                        distance
                    );
                    view_dist.push(distance);
                }
                camera_dist.push(view_dist);
            }
            distances.push(camera_dist);
        }

        AngleDistanceCache { distances, params }
    }
}
