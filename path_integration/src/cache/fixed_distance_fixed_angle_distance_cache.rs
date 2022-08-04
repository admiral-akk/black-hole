use crate::{cast_ray_steps_response, find_optimal_z, structs::response::Response};

use serde::{Deserialize, Serialize};

fn find_z_bounds_for_angle(
    camera_distance: f64,
    black_hole_radius: f64,
    epsilon: f64,
    distance_bounds: (f64, f64),
    target_angle: f64,
) -> (f64, f64) {
    let bound_predicate = |r: Response| r.hits_black_hole();
    let valid_z = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (-1., 1.),
        &bound_predicate,
    );

    let is_too_close = move |r: Response| {
        let angle_d = r.get_angle_dist();
        if angle_d.get_max_angle() < target_angle {
            return false;
        }
        if angle_d.get_dist(target_angle).unwrap() > distance_bounds.1 {
            return false;
        }
        true
    };
    let lower = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (-1., valid_z.0),
        &is_too_close,
    );
    let is_too_close = move |r: Response| {
        let angle_d = r.get_angle_dist();
        if angle_d.get_max_angle() < target_angle {
            return true;
        }
        if angle_d.get_dist(target_angle).unwrap() < distance_bounds.0 {
            return true;
        }
        false
    };
    let upper = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (valid_z.0, 1.),
        &is_too_close,
    );

    (lower.1, upper.0)
}

const Z_EPSILON: f64 = 0.000000001;
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FixedDistanceFixedAngleDistanceCache {
    pub camera_distance: f64,
    pub black_hole_radius: f64,
    pub disc_bounds: (f64, f64),
    pub z_bounds: (f64, f64),
    pub angle: f64,
    pub z_to_distance: Vec<f64>,
}

fn float_01_to_left_index(float_01: f64, vec_len: usize) -> (usize, f64) {
    let float_index = (vec_len - 1) as f64 * float_01;
    let mut index = float_index as usize;
    if index == vec_len - 1 {
        index -= 1;
    }
    let t = float_index - index as f64;
    (index, t)
}
fn index_to_float_01(index: usize, vec_len: usize) -> f64 {
    (index as f64) / (vec_len - 1) as f64
}
impl FixedDistanceFixedAngleDistanceCache {
    pub fn compute_new(
        cache_size: usize,
        camera_distance: f64,
        black_hole_radius: f64,
        disc_bounds: (f64, f64),
        angle: f64,
    ) -> Self {
        let z_bounds = find_z_bounds_for_angle(
            camera_distance,
            black_hole_radius,
            Z_EPSILON,
            disc_bounds,
            angle,
        );

        let mut z_to_distance = Vec::new();
        for i in 0..cache_size {
            let float_01 = index_to_float_01(i, cache_size);
            let z = (z_bounds.1 - z_bounds.0) * float_01 + z_bounds.0;
            let response =
                cast_ray_steps_response(z, camera_distance as f64, black_hole_radius as f64);
            let angle_path = response.get_angle_dist();
            let dist = angle_path.get_dist(angle);
            if dist.is_none() {
                panic!(
                    "Should always hit angle!\nz_range: {:?}\nangle: {}\nz: {}\nmax_angle: {}\nfinal_dist: {}\n",
                    z_bounds,
                    angle,
                    z,
                    angle_path.get_max_angle(),
                    angle_path.get_final_dist(),
                )
            } else {
                let dist = dist.unwrap();
                z_to_distance.push(dist);
            }
        }
        FixedDistanceFixedAngleDistanceCache {
            camera_distance,
            black_hole_radius,
            disc_bounds,
            z_bounds,
            angle,
            z_to_distance,
        }
    }

    pub fn get_dist(&self, float_01: f64) -> f64 {
        let (index, t) = float_01_to_left_index(float_01, self.z_to_distance.len());
        let left = self.z_to_distance[index];
        let right = self.z_to_distance[index + 1];
        right * t + (1. - t) * left
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{PI, TAU};

    use test_utils::plot_trajectories;

    use crate::cast_ray_steps_response;

    use super::FixedDistanceFixedAngleDistanceCache;
    #[test]
    fn fixed_angle_test_error() {
        let cache_size = 16;
        let distance = 10.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (3.0, 6.0);
        let mut lines = Vec::new();

        let angle_iterations = 32;
        let distance_iterations = 512;
        for j in 0..=angle_iterations {
            let mut line = Vec::new();
            let angle = TAU * (j as f64) / (angle_iterations as f64);
            let cache = FixedDistanceFixedAngleDistanceCache::compute_new(
                cache_size,
                distance,
                black_hole_radius,
                max_disc_radius,
                angle,
            );
            for i in 0..=distance_iterations {
                let float_index_01 = (i as f64) / (distance_iterations as f64);
                let approx_dist = cache.get_dist(float_index_01);
                let z = (cache.z_bounds.1 - cache.z_bounds.0) * float_index_01 + cache.z_bounds.0;
                let true_path =
                    cast_ray_steps_response(z, cache.camera_distance, cache.black_hole_radius)
                        .get_angle_dist();
                assert!(
                    true_path.get_max_angle() >= cache.angle,
                    "\nTrue path is too shallow!\nMax angle: {}\nCache angle: {}\nz: {}",
                    true_path.get_max_angle(),
                    cache.angle,
                    z
                );
                let true_dist = true_path.get_dist(cache.angle).unwrap();
                println!(
                    "Angle: {}, data: {:?}",
                    angle,
                    (
                        float_index_01 as f32,
                        (true_dist - approx_dist).abs() as f32,
                    )
                );
                line.push((
                    float_index_01 as f32,
                    (true_dist - approx_dist).abs() as f32,
                ));
            }
            lines.push(line);
        }
        plot_trajectories(
            "output/angle_cache/fixed_angle_error_rates.png",
            &lines,
            ((0., 1.), (0., 0.1)),
        )
        .unwrap();
    }

    #[test]
    fn serialization() {
        let cache_size = 512;
        let distance = 10.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (3.0, 6.0);
        let angle = PI;
        let cache = FixedDistanceFixedAngleDistanceCache::compute_new(
            cache_size,
            distance,
            black_hole_radius,
            max_disc_radius,
            angle,
        );

        let serialized = serde_json::to_string(&cache);

        assert!(serialized.is_ok());

        let deserialized: Result<FixedDistanceFixedAngleDistanceCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, cache);
    }
}
