use glam::{DVec3};

use crate::{
    cast_ray_steps_response, find_bound_with_grazing_distance,
    find_optimal_z,
    structs::{response::Response, utils::PolarAngle},
    Field,
};

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

use super::fixed_distance_ray_cache::RAY_START_DIR;

const Z_EPSILON: f64 = 0.000000001;
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FixedDistanceAngleCache {
    pub cache: Vec<FixedAngleFixedDistanceAngleCache>,
    pub min_z: f32,
    pub distance: f32,
}
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FixedAngleFixedDistanceAngleCache {
    pub angle: f32,
    pub z_range: (f32, f32),
    pub angle_dist: Vec<AngleCacheAnswer>,
}
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AngleCacheAnswer {
    pub angle: f32,
    pub dist: f32,
    pub z: f32,
}

const POW: f64 = 5.;
const MAX_Z: f64 = 0.99999;

fn index_to_z(index: usize, size: usize, min_z: f64) -> f64 {
    let mut t = (index as f64) / ((size - 1) as f64);
    let c = 0.5_f64.powf(1. - POW);
    if t > 0.5 {
        t = 1. - c * (1. - t).powf(POW);
    } else {
        t = c * t.powf(POW);
    }
    let z = MAX_Z - (MAX_Z - min_z) * t;
    z
}

fn z_to_index(z: f32, size: usize, min_z: f64) -> f32 {
    let mut t = (z as f64 - MAX_Z) / (min_z - MAX_Z);
    let c = 0.5_f64.powf(1. - POW);
    if t > 0.5 {
        t = 1. - ((1. - t) / c).powf(1. / POW);
    } else {
        t = (t / c).powf(1. / POW);
    }
    ((size - 1) as f32) * t as f32
}

fn to_angle_dist(pos: &DVec3) -> (f32, f32) {
    return (pos.get_angle() as f32, pos.length() as f32);
}
fn find_z_bounds_for_angle(
    camera_distance: f32,
    black_hole_radius: f32,
    epsilon: f64,
    distance_bounds: (f64, f64),
    target_angle: f64,
) -> (f64, f64) {
    println!("\n\ntarget angle: {:?}", target_angle);
    let bound_predicate = |r: Response| r.hits_black_hole();
    let valid_z = find_optimal_z(
        camera_distance,
        black_hole_radius,
        epsilon,
        (-1., 1.),
        &bound_predicate,
    );

    println!("valid_z: {:?}", valid_z);
    let z = valid_z.0;
    println!(
        "max_angle: {:?}",
        cast_ray_steps_response(z, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_max_angle()
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
        camera_distance,
        black_hole_radius,
        epsilon,
        (-1., valid_z.0),
        &is_too_close,
    );
    println!("lower: {:?}", lower);

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
        camera_distance,
        black_hole_radius,
        epsilon,
        (valid_z.0, 1.),
        &is_too_close,
    );
    println!("upper: {:?}", upper);

    (lower.1, upper.0)
}

impl FixedAngleFixedDistanceAngleCache {
    pub fn compute_new(
        cache_dimensions: usize,
        black_hole_radius: f32,
        camera_distance: f32,
        disc_bounds: (f32, f32),
        angle: f32,
    ) -> Self {
        let z_range = find_z_bounds_for_angle(
            camera_distance,
            black_hole_radius,
            Z_EPSILON,
            (disc_bounds.0 as f64, disc_bounds.1 as f64),
            angle as f64,
        );

        let mut angle_dist = Vec::new();
        for i in 0..cache_dimensions {
            let z = (z_range.1 - z_range.0) * (i as f64) / (cache_dimensions - 1) as f64
                + z_range.0 as f64;
            let response =
                cast_ray_steps_response(z, camera_distance as f64, black_hole_radius as f64);
            let angle_path = response.get_angle_dist();
            let dist = angle_path.get_dist(angle as f64);
            if dist.is_none() {
                panic!(
                    "Should always hit angle!\nz_range: {:?}\nangle: {}\nz: {}\nmax_angle: {}\nfinal_dist: {}\n",
                    z_range,
                    angle,
                    z,
                    angle_path.get_max_angle(),
                    angle_path.get_final_dist(),
                )
            } else {
                let z = z as f32;
                let dist = dist.unwrap() as f32;
                angle_dist.push(AngleCacheAnswer { angle, dist, z })
            }
        }
        let z_range = (z_range.0 as f32, z_range.1 as f32);
        FixedAngleFixedDistanceAngleCache {
            angle,
            z_range,
            angle_dist,
        }
    }

    pub fn get_dist(&self, z: f32) -> Option<f32> {
        let z_range = self.z_range;
        if z_range.0 > z || z_range.1 < z {
            return None;
        }
        let mut index = ((self.angle_dist.len() - 1) as f32 * (z - z_range.0)
            / (z_range.1 - z_range.0)) as usize;
        if index == self.angle_dist.len() - 1 {
            index -= 1;
        }
        let left = &self.angle_dist[index];
        let right = &self.angle_dist[index + 1];
        let t = (z - left.z) / (right.z - left.z);
        return Some(t * right.dist + (1. - t) * left.dist);
    }
}

impl FixedDistanceAngleCache {
    pub fn compute_new(
        cache_dimensions: (usize, usize),
        black_hole_radius: f32,
        camera_distance: f32,
        disc_bounds: (f32, f32),
    ) -> Self {
        let mut cache = Vec::new();

        let cache_pos = camera_distance * RAY_START_DIR;

        let field = Field::new(black_hole_radius as f64, camera_distance as f64);
        let ring_radius = disc_bounds.1 as f64;
        let ray_travel_distance_limit = 10.0 * camera_distance as f64;

        let bound = find_bound_with_grazing_distance(
            &cache_pos,
            &field,
            Z_EPSILON,
            ray_travel_distance_limit,
            ring_radius,
        );

        for i in 0..cache_dimensions.0 {
            let angle = TAU * (i as f32) / (cache_dimensions.0 - 1) as f32;
            cache.push(FixedAngleFixedDistanceAngleCache::compute_new(
                cache_dimensions.1,
                black_hole_radius,
                camera_distance,
                disc_bounds,
                angle,
            ));
        }
        Self {
            cache,
            min_z: bound as f32,
            distance: camera_distance,
        }
    }

    pub fn get_dist(&self, z: f32, angle: f32) -> Option<f32> {
        let _float_index = angle / TAU;

        if z < self.min_z {
            return None;
        }
        let mut index = ((self.cache.len() - 1) as f32 * angle / TAU) as usize;
        if index == self.cache.len() - 1 {
            index -= 1;
        }
        let (low, high) = (&self.cache[index], &self.cache[index + 1]);

        let t = (angle - low.angle) / (high.angle - low.angle);

        let (low, high) = (low.get_dist(z), high.get_dist(z));
        if low.is_none() {
            return None;
        }
        if high.is_some() {
            return Some(t * high.unwrap() + (1. - t) * low.unwrap());
        }
        return low;
    }
}

#[cfg(test)]
mod tests {
    
    use test_utils::plot_trajectories;

    

    use crate::cache::angle_cache::TestStats;
    use crate::cast_ray_steps_response;
    use crate::{
        cache::fixed_distance_angle_cache::FixedDistanceAngleCache,
    };

    
    #[test]
    fn render_index_to_max_angle() {
        let cache_size = (512, 512);
        let black_hole_radius = 1.5;
        let max_disc_radius = (3.0, 6.0);
        let distance = 10.0;
        let cache = FixedDistanceAngleCache::compute_new(
            cache_size,
            black_hole_radius,
            distance,
            max_disc_radius,
        );
        let mut lines = Vec::new();
        let mut line = Vec::new();
        for i in 0..cache.cache.len() {
            let inner_cache = &cache.cache[i];
            for _ in 0..inner_cache.angle_dist.len() {
                let angle_dist = &inner_cache.angle_dist;
                let mut j = 0;
                while j < angle_dist.len() - 1 && angle_dist[j].dist > angle_dist[j + 1].dist {
                    j += 1;
                }
                line.push((
                    i as f32 / (cache.cache.len() - 1) as f32,
                    angle_dist[j].angle,
                ));
            }
        }
        lines.push(line);
        plot_trajectories(
            "output/angle_cache/index_to_max_angle.png",
            &lines,
            ((0., 1.), (0., 2. * std::f64::consts::TAU)),
        );
    }

    #[test]
    fn fixed_dist_angle_close() {
        let cache_size = (256, 64);
        let black_hole_radius = 1.5;
        let max_disc_radius = (3.0, 6.0);
        let distance = 10.0;
        let cache = FixedDistanceAngleCache::compute_new(
            cache_size,
            black_hole_radius,
            distance,
            max_disc_radius,
        );

        let iterations = 100;
        for x in 0..=iterations {
            let x = (x as f64) / (iterations as f64);
            let z = (1. - x * x).sqrt();
            if z > 0.9999 {
                continue;
            }
            let path = cast_ray_steps_response(z, distance as f64, black_hole_radius as f64)
                .get_angle_dist();

            let mut test_stats = TestStats::default();
            let mut false_positive = 0;
            let mut false_negative = 0;
            let mut true_negative = 0;
            let mut true_positive = 0;

            for angle in 1..360 {
                let angle = std::f64::consts::TAU * angle as f64 / 360.0;
                let approx_dist = cache.get_dist(z as f32, angle as f32);
                if path.get_max_angle() < angle || z < cache.min_z as f64 {
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
                        distance,
                        angle as f32,
                        true_dist,
                        approx_dist.unwrap(),
                    );
                }
            }

            println!("\n\nStats for z: {}\n\n", z);
            println!("false_positives: {}", false_positive);
            println!("false_negative: {}", false_negative);
            println!("true_negative: {}", true_negative);
            println!("true_positive: {}", true_positive);
            println!("test stats: {:?}", test_stats);
        }
    }
    #[test]
    fn serialization() {
        let distance = 10.0;
        let max_disc_radius = (3.0, 6.0);
        let r = 1.5;
        let cache_size = (5, 5);
        let angle_cache =
            FixedDistanceAngleCache::compute_new(cache_size, r, distance, max_disc_radius);

        let serialized = serde_json::to_string(&angle_cache);

        assert!(serialized.is_ok());

        let deserialized: Result<FixedDistanceAngleCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, angle_cache);
    }
}
