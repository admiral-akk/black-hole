use std::fs;

use wire_structs::{
    angle_distance_cache::{generate_particles, AngleDistanceCacheParams, DimensionParams},
    gpu::gpu_state::simulate_particles,
    path_integration::path::cast_ray_steps_response,
};

use super::angle_test_point::AngleTestPoint;

pub fn regenerate_angle_distance_test_points(
    params: &AngleDistanceCacheParams,
) -> Vec<AngleTestPoint> {
    let folder_path = &format!(
        "generate_artifacts/output/angle_test_points_{}",
        params.test_name(),
    );
    fs::create_dir_all(folder_path).unwrap();
    let path = format!("{}/points.txt", folder_path);
    let data = fs::read(&path);
    if data.is_ok() {
        println!("Found existing test set!");
        return serde_json::from_slice(&data.unwrap()).unwrap();
    }

    let dists = DimensionParams {
        size: 15,
        bounds: params.dist.bounds,
    };
    let views = DimensionParams {
        size: 4000,
        bounds: params.view_dist.bounds,
    };
    let angles = DimensionParams {
        size: 61,
        bounds: params.angle.bounds,
    }
    .generate_list();

    let particles = generate_particles(&dists, &views, params);
    let result = simulate_particles(particles, 61);
    let mut test_points = Vec::new();
    let dists = dists.generate_list();
    let views = views.generate_list();
    for (dist_index, dist) in dists.iter().enumerate() {
        let result = &result[dist_index * views.len()..(dist_index + 1) * views.len()];
        for (view_index, view) in views.iter().enumerate() {
            println!(
                "Generating test case ({}/{}, {}/{})",
                dist_index + 1,
                dists.len(),
                view_index + 1,
                views.len()
            );
            for (angle_index, target_angle) in angles.iter().enumerate() {
                let mut dist_at_angle = Some(result[view_index].angle_dist[angle_index] as f64);
                if dist_at_angle.unwrap() < 1. {
                    dist_at_angle = None;
                }
                test_points.push(AngleTestPoint {
                    view_port_coord: *view as f64,
                    target_angle: *target_angle as f64,
                    dist: *dist as f64,
                    dist_at_angle: dist_at_angle,
                });
            }
        }
    }
    let data = serde_json::to_string(&test_points).unwrap();
    fs::write(&path, data).expect("Unable to write file");
    test_points
}
