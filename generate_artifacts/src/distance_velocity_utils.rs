use std::f32::consts::TAU;

use test_utils::plot_with_title;
use wire_structs::sampler::{
    dimension_params::DimensionParams, distance_velocity_paths::DistanceVelocityPaths,
    simulated_path::SimulatedPath,
};

pub fn analyze_distance_velocity(
    dist_vel: &DistanceVelocityPaths,
    _dist: &DimensionParams,
    angle: &DimensionParams,
) {
    plot_path(&dist_vel.paths, &angle);
}

fn plot_velocity_to_dist(dist: &DimensionParams, velocities: &Vec<f32>) {
    let mut points = Vec::new();
    for (d_i, d) in dist.generate_list().iter().enumerate() {
        points.push((*d, velocities[d_i]));
    }
    let mut lines = Vec::new();
    lines.push(points);
    plot_with_title(
        &format!("Velocity by Distance"),
        &format!(
            "generate_artifacts/output/dist_velocity/{}.png",
            "velocities",
        ),
        &lines,
        ((0., dist.bounds[1] as f64), (0., 1.)),
    )
    .unwrap();
}

fn plot_path(paths: &Vec<SimulatedPath>, _angle: &DimensionParams) {
    println!("Generating {}", "paths");
    let angle = DimensionParams {
        size: 360,
        bounds: [0., TAU],
    };

    let mut plot_line_groups = Vec::new();
    let mut line_group = (paths[0].dist, Vec::new());
    let angles = angle.generate_list();
    for path in paths {
        let mut line = Vec::new();
        for (i, angle) in angles.iter().enumerate() {
            let dist = path.ray.angle_dist[i];
            if dist == 0. {
                break;
            }
            line.push((dist * angle.sin(), -dist * angle.cos()));
        }
        if line_group.0 != path.dist {
            plot_line_groups.push(line_group);
            line_group = (path.dist, Vec::new());
        }
        line_group.1.push(line);
    }
    plot_line_groups.push(line_group);
    for (dist, line_group) in plot_line_groups {
        println!("Generating {}, dist: {}", "paths", dist);
        plot_with_title(
            &format!("{} at dist = {}", "Paths", dist),
            &format!(
                "generate_artifacts/output/dist_velocity/{}/{}_{}.png",
                "path", "path", dist
            ),
            &line_group,
            ((-30., 30.), (-30., 30.)),
        )
        .unwrap();
    }
}
