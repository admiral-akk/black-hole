use crate::{cast_ray_steps_response, find_z_bounds_for_angle};

use serde::{Deserialize, Serialize};

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

fn float_01_to_left_index(mut float_01: f64, vec_len: usize) -> (usize, f64) {
    if float_01 > 0.5 {
        float_01 = 2.0 * (float_01 - 0.5);
        float_01 = float_01 * float_01;
        float_01 = float_01 / 2.0 + 0.5;
    } else {
        float_01 = 2.0 * (0.5 - float_01);
        float_01 = float_01 * float_01;
        float_01 = 0.5 - float_01 / 2.0;
    }
    let float_index = (vec_len - 1) as f64 * float_01;
    let mut index = float_index as usize;
    if index == vec_len - 1 {
        index -= 1;
    }
    let t = float_index - index as f64;
    (index, t)
}
fn index_to_float_01(index: usize, vec_len: usize) -> f64 {
    let mut float_01 = (index as f64) / (vec_len - 1) as f64;
    if float_01 > 0.5 {
        float_01 = (2. * (float_01 - 0.5)).sqrt() / 2. + 0.5;
    } else {
        float_01 = 0.5 - (2. * (0.5 - float_01)).sqrt() / 2.;
    }
    float_01
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
            let response = cast_ray_steps_response(z, camera_distance, black_hole_radius);
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
                if i == 0 {
                    assert!(
                        (dist - disc_bounds.1).abs() < 0.1,
                        "First ray doesn't hit outer edge!\nActual dist: {}\nDisc bounds: {:?}\n",
                        dist,
                        disc_bounds
                    );
                } else if i == cache_size - 1 {
                    assert!(
                        (dist - disc_bounds.0).abs() < 0.1,
                        "Last ray doesn't hit inner edge!\nActual dist: {}\nDisc bounds: {:?}\n",
                        dist,
                        disc_bounds
                    );
                }
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

    pub fn get_dist(&self, z_01: f64) -> f64 {
        let (index, t) = float_01_to_left_index(z_01, self.z_to_distance.len());
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
        let cache_size = 256;
        let distance = 17.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (3.0, 6.0);
        let mut lines = Vec::new();

        let angle_iterations = 256;
        let distance_iterations = 1024;
        for j in 1..=(angle_iterations / 10) {
            let mut line = Vec::new();
            let angle = TAU * (j as f64) / (angle_iterations as f64);
            let cache = FixedDistanceFixedAngleDistanceCache::compute_new(
                cache_size,
                distance,
                black_hole_radius,
                max_disc_radius,
                angle,
            );
            for i in 0..=(distance_iterations / 10) {
                let z_01 = (i as f64) / (distance_iterations as f64);
                let approx_dist = cache.get_dist(z_01);
                let z = (cache.z_bounds.1 - cache.z_bounds.0) * z_01 + cache.z_bounds.0;
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
                    (z_01 as f32, (true_dist - approx_dist).abs() as f32,)
                );
                line.push((z_01 as f32, (true_dist - approx_dist).abs() as f32));
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
