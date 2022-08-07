use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::path_integration2::{
    path::cast_ray_steps_response, path::find_optimal_z, response::Response,
};

const MIN_ANGLE: f64 = TAU * (0.1 / 360.);
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

const LINEAR_INDEX_WEIGHT: f64 = 0.5;

fn float_01_to_left_index(mut float_01: f64, vec_len: usize) -> (usize, f64) {
    let float_02 = float_01;
    if float_01 > 0.5 {
        float_01 = 2.0 * (float_01 - 0.5);
        float_01 = float_01 * float_01;
        float_01 = float_01 / 2.0 + 0.5;
    } else {
        float_01 = 2.0 * (0.5 - float_01);
        float_01 = float_01 * float_01;
        float_01 = 0.5 - float_01 / 2.0;
    }
    float_01 = float_02.sqrt();
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
    float_01 = float_01 * float_01;
    return float_01;
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
                        "First ray doesn't hit outer edge!\nActual dist: {}\nDisc bounds: {:?}\nangle: {}\nz: {}\n",
                        dist,
                        disc_bounds,
                        angle,
                        z,
                    );
                } else if i == cache_size - 1 {
                    assert!(
                        (dist - disc_bounds.0).abs() < 0.1,
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
    println!("valid z: {:?}", valid_z);

    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        return dist.is_some() && dist.unwrap() <= distance_bounds.1;
    };
    let lower_1 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (-1., valid_z.0),
        &is_too_close,
    );
    println!("Lower 1: {:?}", lower_1);
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
        epsilon,
        (valid_z.0, 1.0),
        &is_too_close,
    );
    println!("Lower 2: {:?}", lower_2);
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
        epsilon,
        (-1., valid_z.0),
        &is_too_close,
    );
    println!("Lower 2: {:?}", upper_1);
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
        epsilon,
        (valid_z.0, 1.0),
        &is_too_close,
    );
    println!("Lower 2: {:?}", upper_2);
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
    println!("Bounds: {:?}", bounds);

    bounds
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};

    use test_utils::plot_trajectories;

    use crate::{
        path_distance_cache::fixed_distance_fixed_angle_distance_cache::index_to_float_01,
        path_integration2::path::cast_ray_steps_response,
    };

    use super::{FixedDistanceFixedAngleDistanceCache, MIN_ANGLE};
    #[test]
    fn fixed_angle_test_error() {
        let cache_size = 1 << 12;
        let distance = 3.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (1.5, 12.0);
        let mut lines = Vec::new();

        let mut angle = MIN_ANGLE;
        while angle < TAU {
            let mut line = Vec::new();
            let cache = FixedDistanceFixedAngleDistanceCache::compute_new(
                cache_size,
                distance,
                black_hole_radius,
                max_disc_radius,
                angle,
            );
            for i in 0..(cache_size - 1) {
                let i_01 = index_to_float_01(i, cache_size);
                let z_01 = i_01 + 1. / (2 * cache_size) as f64;
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
                if (true_dist - approx_dist).abs() > 0.1 {
                    println!(
                        "Angle: {}, data: {:?}",
                        angle,
                        (z_01 as f32, (true_dist - approx_dist).abs() as f32,)
                    );
                }
                line.push((z_01 as f32, (true_dist - approx_dist).abs() as f32));
            }
            lines.push(line);

            angle *= 2.0;
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
