use serde::{Deserialize, Serialize};

use crate::{
    final_direction_cache::direction_cache::DirectionCache,
    path_distance_cache::distance_cache::DistanceCache,
};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct BlackHoleCache<T> {
    pub direction_cache_size: (usize, usize),
    pub distance_bounds: (T, T),
    pub black_hole_radius: T,
    pub distance_cache_size: (usize, usize, usize),
    pub disc_bounds: (T, T),
    pub distance_cache: DistanceCache<T>,
    pub direction_cache: DirectionCache<T>,
}

impl BlackHoleCache<f64> {
    pub fn compute_new(
        distance_bounds: (f64, f64),
        black_hole_radius: f64,
        disc_bounds: (f64, f64),
        direction_cache_size: (usize, usize),
        distance_cache_size: (usize, usize, usize),
    ) -> Self {
        let distance_cache = DistanceCache::compute_new(
            distance_cache_size,
            distance_bounds,
            black_hole_radius,
            disc_bounds,
        );
        let direction_cache =
            DirectionCache::compute_new(direction_cache_size, distance_bounds, black_hole_radius);
        BlackHoleCache {
            direction_cache_size,
            distance_bounds,
            black_hole_radius,
            distance_cache_size,
            disc_bounds,
            distance_cache,
            direction_cache,
        }
    }

    pub fn new(distance_cache: DistanceCache<f64>, direction_cache: DirectionCache<f64>) -> Self {
        assert!(direction_cache.black_hole_radius == distance_cache.black_hole_radius);
        assert!(direction_cache.distance_bounds == distance_cache.distance_bounds);
        BlackHoleCache {
            direction_cache_size: direction_cache.cache_size,
            distance_bounds: direction_cache.distance_bounds,
            black_hole_radius: direction_cache.black_hole_radius,
            distance_cache_size: distance_cache.cache_size,
            disc_bounds: distance_cache.disc_bounds,
            distance_cache,
            direction_cache,
        }
    }
}
