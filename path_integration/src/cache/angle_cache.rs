use std::f32::consts::E;

use serde::{Deserialize, Serialize};

use super::fixed_distance_angle_cache::FixedDistanceAngleCache;

#[derive(Debug, PartialEq, Default)]
pub struct TestStats {
    pub dist: f32,
    pub z: f32,
    pub angle: f32,
    pub miss: u32,
    pub sum_sq_error: f32,
    pub sample_count: f32,
    pub rms_total: f32,
    pub max_error: f32,
    pub max_error_angle: f32,
    pub max_error_dist: f32,
    pub max_error_z: f32,
}

impl TestStats {
    pub fn add_sample(&mut self, z: f32, dist: f32, angle: f32, true_val: f32, approx_val: f32) {
        let error = (true_val - approx_val).abs();
        if error > self.max_error {
            self.max_error = error;
            self.max_error_angle = angle;
            self.max_error_dist = dist;
            self.max_error_z = z;
        }
        self.sum_sq_error += error * error;
        self.sample_count += 1.;
        self.rms_total = self.sum_sq_error.sqrt() / self.sample_count;
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AngleCache {
    pub cache: Vec<FixedDistanceAngleCache>,
    pub distance_bounds: (f32, f32),
}

fn dist_to_index(distance: f32, size: usize, distance_bounds: (f32, f32)) -> f32 {
    ((size - 1) as f32)
        * ((distance - distance_bounds.0) / (distance_bounds.1 - distance_bounds.0)) as f32
}
impl AngleCache {
    pub fn compute_new(
        cache_dimensions: (usize, usize, usize),
        black_hole_radius: f32,
        distance_bounds: (f32, f32),
        max_disc_radius: (f32, f32),
    ) -> Self {
        let mut cache = Vec::new();
        for i in 0..cache_dimensions.0 {
            // let distance = (distance_bounds.1 - distance_bounds.0)
            //     * (i as f32 / (cache_dimensions.0 - 1) as f32)
            //     + distance_bounds.0;
            let distance = 17.0;
            cache.push(FixedDistanceAngleCache::compute_new(
                (cache_dimensions.1, cache_dimensions.2),
                black_hole_radius,
                distance,
                max_disc_radius,
            ));
        }
        Self {
            cache,
            distance_bounds,
        }
    }

    pub fn get_dist(&self, distance: f32, z: f32, angle: f32) -> Option<f32> {
        let float_index =
            (distance - self.distance_bounds.0) / (self.distance_bounds.1 - self.distance_bounds.0);
        let mut index = dist_to_index(distance, self.cache.len(), self.distance_bounds) as usize;
        if index == self.cache.len() - 1 {
            index -= 1;
        }
        let (left, right) = (&self.cache[index], &self.cache[index + 1]);
        let t = (distance - left.distance) / (right.distance - left.distance);

        let (left, right) = (left.get_dist(z, angle), right.get_dist(z, angle));
        if left.is_none() {
            return None;
        }
        if right.is_some() {
            return Some(t * right.unwrap() + (1. - t) * left.unwrap());
        }
        return left;
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::{
        cache::angle_cache::{AngleCache, TestStats},
        cast_ray_steps_response,
    };

    #[test]
    fn all_dist_angle_close() {
        let cache_size = (16, 256, 64);
        let black_hole_radius = 1.5;
        let max_disc_radius = (3.0, 6.0);
        let distance = (7.0, 20.0);
        let cache =
            serde_json::from_str::<AngleCache>(&fs::read_to_string("angle_cache.txt").unwrap())
                .unwrap();
        // AngleCache::compute_new(cache_size, black_hole_radius, distance, max_disc_radius);

        let iterations = 100;
        for d in 0..=iterations {
            let dist = (d as f32 / iterations as f32) * (distance.1 - distance.0) + distance.0;
            for x in 0..=iterations {
                let x = (x as f64) / (iterations as f64);
                let z = (1. - x * x).sqrt();
                if z > 0.9999 {
                    continue;
                }
                let path = cast_ray_steps_response(z, dist as f64, black_hole_radius as f64)
                    .get_angle_dist();

                let mut test_stats = TestStats::default();
                let mut false_positive = 0;
                let mut false_negative = 0;
                let mut true_negative = 0;
                let mut true_positive = 0;

                for angle in 1..360 {
                    let angle = std::f64::consts::TAU * angle as f64 / 360.0;
                    let approx_dist = cache.get_dist(dist, z as f32, angle as f32);
                    if path.get_max_angle() < angle {
                        true_negative += 1;
                        if approx_dist.is_some() {
                            false_positive += 1;
                        }
                        continue;
                    }
                    true_positive += 1;
                    let true_dist = path.get_dist(angle);
                    assert!(
                        true_dist.is_some(),
                        "Angle is missing. Angle: {}, z: {}",
                        angle,
                        z
                    );
                    let true_dist = true_dist.unwrap() as f32;
                    if approx_dist.is_none() {
                        false_negative += 1;
                    } else {
                        test_stats.add_sample(
                            z as f32,
                            dist,
                            angle as f32,
                            true_dist,
                            approx_dist.unwrap(),
                        );
                    }
                }

                println!("\n\nStats for z: {}, dist: {}\n\n", z, dist);
                println!("false_positives: {}", false_positive);
                println!("false_negative: {}", false_negative);
                println!("true_negative: {}", true_negative);
                println!("true_positive: {}", true_positive);
                println!("test stats: {:?}", test_stats);
            }
        }
    }

    #[test]
    fn serialization() {
        let distance_bounds = (7.0, 20.0);
        let max_disc_radius = (3.0, 6.0);
        let r = 1.0;
        let cache_size = (4, 8, 16);
        let angle_cache = AngleCache::compute_new(cache_size, r, distance_bounds, max_disc_radius);

        let serialized = serde_json::to_string(&angle_cache);

        assert!(serialized.is_ok());

        let deserialized: Result<AngleCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, angle_cache);
    }
}
