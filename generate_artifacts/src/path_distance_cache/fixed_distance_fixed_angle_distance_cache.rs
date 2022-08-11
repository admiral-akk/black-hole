use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::path_integration2::{
    path::cast_ray_steps_response, path::find_optimal_z, response::Response,
};

pub const MIN_ANGLE: f64 = TAU * (0.001 / 360.);
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FixedDistanceFixedAngleDistanceCache {
    pub camera_distance: f64,
    pub black_hole_radius: f64,
    pub disc_bounds: (f64, f64),
    pub z_bounds: (f64, f64),
    pub angle: f64,
    pub z_to_distance: Vec<f64>,
}

fn float_01_to_index_01(float_01: f64) -> f64 {
    return float_01.clamp(0., 1.);
}

fn float_01_to_left_index(float_01: f64, vec_len: usize) -> (usize, f64) {
    let float_index = (vec_len - 1) as f64 * float_01_to_index_01(float_01);
    let index = (float_index as usize).clamp(0, vec_len - 2);
    let t = float_index - index as f64;
    (index, t)
}
fn index_to_float_01(index: usize, vec_len: usize) -> f64 {
    let float_01 = (index as f64) / (vec_len - 1) as f64;
    return float_01.clamp(0., 1.);
}

impl FixedDistanceFixedAngleDistanceCache {
    pub fn compute_new(
        cache_size: usize,
        camera_distance: f64,
        black_hole_radius: f64,
        disc_bounds: (f64, f64),
        angle: f64,
    ) -> Self {
        let z_bounds =
            find_z_bounds_for_angle(camera_distance, black_hole_radius, disc_bounds, angle);

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
                if i > cache_size - 1 {
                    assert!(
                        (dist - disc_bounds.0).abs() < 0.1,
                        "First ray doesn't hit outer edge!\nActual dist: {}\nDisc bounds: {:?}\nangle: {}\nz: {}\n",
                        dist,
                        disc_bounds,
                        angle,
                        z,
                    );
                } else if i < 0 {
                    assert!(
                        (dist - disc_bounds.1).abs() < 0.1,
                        "Last ray doesn't hit inner edge!\nActual dist: {}\nDisc bounds: {:?}\nangle: {}\nz: {}\n",
                        dist,
                        disc_bounds,
                        angle,
                        z,
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

fn find_z_bounds_for_angle(
    camera_distance: f64,
    black_hole_radius: f64,
    distance_bounds: (f64, f64),
    target_angle: f64,
) -> (f64, f64) {
    let bound_predicate = |r: Response| r.hits_black_hole();
    let valid_z = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        (-1., 1.),
        &bound_predicate,
    );

    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        return dist.is_some() && dist.unwrap() <= distance_bounds.1;
    };
    let lower_1 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        (-1., valid_z.0),
        &is_too_close,
    );
    let lower_test_1 =
        cast_ray_steps_response(lower_1.1, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_dist(target_angle)
            .unwrap_or(100.);
    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        dist.is_none() || dist.unwrap() <= distance_bounds.1
    };
    let lower_2 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        (valid_z.0, 1.0),
        &is_too_close,
    );
    let lower_test_2 =
        cast_ray_steps_response(lower_2.1, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_dist(target_angle)
            .unwrap_or(100.);

    let mut lower = lower_1.1;
    if (lower_test_2 - distance_bounds.1).abs() < (lower_test_1 - distance_bounds.1).abs() {
        lower = lower_2.1;
    }

    let _is_too_close = move |r: Response| {
        let angle_d = r.get_angle_dist();
        if angle_d.get_max_angle() < target_angle {
            return true;
        }
        if angle_d.get_dist(target_angle).unwrap() < distance_bounds.0 {
            return true;
        }
        false
    };

    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        return !dist.is_none() && dist.unwrap() <= distance_bounds.0;
    };
    let upper_1 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        (-1., valid_z.0),
        &is_too_close,
    );
    let upper_test_1 =
        cast_ray_steps_response(upper_1.1, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_dist(target_angle)
            .unwrap_or(100.);
    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        dist.is_none() || dist.unwrap() < distance_bounds.0
    };
    let upper_2 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        (valid_z.0, 1.0),
        &is_too_close,
    );
    let upper_test_2 =
        cast_ray_steps_response(upper_2.1, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_dist(target_angle)
            .unwrap_or(100.);

    let mut upper = upper_1.0;
    if (upper_test_2 - distance_bounds.0).abs() < (upper_test_1 - distance_bounds.0).abs() {
        upper = upper_2.0;
    }

    let bounds = (lower, upper);

    bounds
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};

    use test_utils::plot_trajectories;

    use crate::path_integration2::path::cast_ray_steps_response;

    use super::{float_01_to_index_01, FixedDistanceFixedAngleDistanceCache, MIN_ANGLE};

    #[test]
    fn show_index_distribution() {
        let point_count = 1000;
        let mut lines = Vec::new();
        let mut line = Vec::new();
        for i in 0..point_count {
            let z_01 = i as f64 / (point_count - 1) as f64;
            let i_01 = float_01_to_index_01(z_01);

            line.push((z_01 as f32, i_01 as f32));
        }
        lines.push(line);
        plot_trajectories(
            "output/angle_cache/index_distribution.png",
            &lines,
            ((0., 1.), (0., 1.)),
        )
        .unwrap();
    }

    #[test]
    fn fixed_angle_test_error() {
        let cache_size = 1 << 9;
        let distance = 2.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (1.5, 12.0);
        let mut lines = Vec::new();

        let mut samples = Vec::new();
        for i in 0..(2 * cache_size) {
            let z_01 = i as f64 / (2 * cache_size - 1) as f64;
            samples.push(z_01);
            if z_01 > 0. && z_01 < 1. {
                samples.push(z_01 * z_01);
                samples.push(z_01.sqrt());
            }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for angle in [MIN_ANGLE, FRAC_PI_2, PI, TAU] {
            let mut line = Vec::new();
            let cache = FixedDistanceFixedAngleDistanceCache::compute_new(
                cache_size,
                distance,
                black_hole_radius,
                max_disc_radius,
                angle,
            );
            for z_01 in &samples {
                let approx_dist = cache.get_dist(*z_01);
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
                if (true_dist - approx_dist).abs() > 0.1 {
                    println!(
                        "Angle: {}, data: {:?}",
                        angle,
                        (*z_01 as f32, (true_dist - approx_dist).abs() as f32,)
                    );
                }
                line.push((*z_01 as f32, (true_dist - approx_dist).abs() as f32));
            }
            lines.push(line);
        }
        plot_trajectories(
            "output/angle_cache/fixed_angle_error_rates.png",
            &lines,
            ((0., 1.), (0., 0.4)),
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
