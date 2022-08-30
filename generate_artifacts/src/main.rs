use std::f64::consts::TAU;

use std::fs::{self};

use approximation_utils::analyze_approximations;
use artifact_utils::get_or_generate_file;
use combined_approximation_utils::plot_combined_approximate_ray_analysis;

use path_utils::analyze_paths;
use serde::{Deserialize, Serialize};
use wire_structs::sampler::dimension_params::DimensionParams;

use wire_structs::sampler::render_params::RenderParams;
use wire_structs::sampler::simple_path_generator;
mod factory;
mod final_direction_cache;
mod path_distance_cache;
mod path_integration2;

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectionTestPoint {
    pub z: f64,
    pub dist: f64,
    pub final_angle: Option<f64>,
}

const ALL_VIEW_SAMPLE_PATH: &str = "generate_artifacts/output/artifact/all_view.txt";

mod approximation_utils;
mod artifact_utils;
mod combined_approximation_utils;
mod path_utils;
fn main() {
    let dist = DimensionParams {
        size: 128,
        bounds: [5., 30.],
    };
    let view = DimensionParams {
        size: 256,
        bounds: [0., 0.5_f32.sqrt()],
    };
    let angle = DimensionParams {
        size: 512,
        bounds: [0., TAU as f32],
    };

    let render_params = RenderParams {
        black_hole_radius: 1.5,
        fov_degrees: 60.0,
    };

    let all_paths_sample;
    {
        all_paths_sample = get_or_generate_file(ALL_VIEW_SAMPLE_PATH, &move || {
            simple_path_generator::generate_paths(&dist, &view, &angle, &render_params)
        });
    }
    analyze_paths(&all_paths_sample, &angle);
    // analyze_approximations(&all_paths_sample, &dist, &angle);
}
