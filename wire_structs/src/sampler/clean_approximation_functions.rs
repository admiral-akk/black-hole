use std::f32::consts::FRAC_PI_2;

use super::{approximation_function::ApproximationFunction, simulated_path::SimulatedPath};

pub fn linearize_min_dist(paths: &[SimulatedPath], functions: &mut [ApproximationFunction]) {
    let (max_index, dist) = paths
        .iter()
        .enumerate()
        .map(|(i, p)| (i, p.grazing_distance()))
        .filter(|(_, o)| o.is_some())
        .next()
        .unwrap();
    for i in 0..max_index {
        let i_01 = ((i as f32) / max_index as f32).max(0.0001);

        functions[i].min_distance = dist.unwrap() * (1. - (i_01 * FRAC_PI_2).cos());
        functions[i].theta_min_start = (functions[i].min_distance / paths[0].dist).acos();
    }
}
