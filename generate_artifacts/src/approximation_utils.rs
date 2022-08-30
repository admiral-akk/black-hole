use std::f64::consts::TAU;

use test_utils::plot_with_title;
use wire_structs::sampler::{
    approximation_function::{Approximation, ApproximationFunction},
    dimension_params::DimensionParams,
    simulated_path::SimulatedPath,
};

pub fn analyze_approximations(
    paths: &Vec<SimulatedPath>,
    dist: &DimensionParams,
    angle: &DimensionParams,
) {
    let angles = angle.generate_list();
    let mut approximations = Vec::new();
    for (i, path) in paths.iter().enumerate() {
        if i % 100 == 0 {
            println!("Finding approximation {}/{}", i + 1, paths.len());
        }
        approximations.push(ApproximationFunction::new(path, &angles));
    }
    plot_path(&approximations, dist, angle);
    plot_property_by_step(
        &paths,
        &approximations,
        &|path, approx, angles| {
            let mut plot = Vec::new();
            for (i, angle) in angles.iter().enumerate() {
                let dist = path.ray.angle_dist[i];
                if dist == 0. {
                    break;
                }
                let approx = match approx.get_dist(*angle) {
                    Some(dist) => dist,
                    None => 0.,
                };
                let diff = approx - dist;
                plot.push((*angle, diff));
            }
            plot
        },
        "error",
        "weight",
        "Close weight",
        ((0., TAU), (-10., 10.)),
        angle,
    );
}

fn plot_path(paths: &Vec<ApproximationFunction>, dist: &DimensionParams, angle: &DimensionParams) {
    println!("Generating {}", "paths");
    let dists = dist.generate_list();

    let mut plot_line_groups = Vec::new();
    let mut line_group = (dists[0], Vec::new());
    let angles = angle.generate_list();
    for (i, path) in paths.iter().enumerate() {
        let mut line = Vec::new();
        for (i, angle) in angles.iter().enumerate() {
            let dist = path.get_dist(*angle);
            if dist.is_none() {
                break;
            }
            let dist = dist.unwrap();
            if dist <= 1.5 {
                break;
            }
            line.push((dist * angle.sin(), -dist * angle.cos()));
        }
        if (i + 1) % (paths.len() / dists.len()) == 0 {
            let dist_index = i / (paths.len() / dists.len());
            plot_line_groups.push(line_group);
            line_group = (dists[dist_index], Vec::new());
        }
        line_group.1.push(line);
    }
    plot_line_groups.push(line_group);
    for (dist, line_group) in plot_line_groups {
        println!("Generating {}, dist: {}", "paths", dist);
        plot_with_title(
            &format!("{} at dist = {}", "Paths", dist),
            &format!(
                "generate_artifacts/output/approximation_function/{}/{}_{}.png",
                "true_path", "path", dist
            ),
            &line_group,
            ((-30., 30.), (-30., 30.)),
        )
        .unwrap();
    }
}

fn plot_property_by_step(
    paths: &Vec<SimulatedPath>,
    approximation: &Vec<ApproximationFunction>,
    property: &dyn Fn(&SimulatedPath, &ApproximationFunction, &Vec<f32>) -> Vec<(f32, f32)>,
    file_name: &str,
    folder: &str,
    plot_title: &str,
    bounds: ((f64, f64), (f64, f64)),
    angle: &DimensionParams,
) {
    println!("Generating {}", plot_title);

    let mut plot_line_groups = Vec::new();
    let mut line_group = (paths[0].dist, Vec::new());

    let angles = angle.generate_list();
    for (i, path) in paths.iter().enumerate() {
        let mut line = property(path, &approximation[i], &angles);
        if line_group.0 != path.dist {
            plot_line_groups.push(line_group);
            line_group = (path.dist, Vec::new());
        }
        line_group.1.push(line);
    }
    plot_line_groups.push(line_group);
    for (dist, line_group) in plot_line_groups {
        println!("Generating {}, dist: {}", plot_title, dist);
        plot_with_title(
            &format!("{} at dist = {}", plot_title, dist),
            &format!(
                "generate_artifacts/output/approximation_function/{}/{}_{}.png",
                folder, file_name, dist
            ),
            &line_group,
            bounds,
        )
        .unwrap();
    }
}

fn plot_property_by_path(
    paths: &Vec<SimulatedPath>,
    property: &dyn Fn(&SimulatedPath) -> (f32, f32),
    file_name: &str,
    folder: &str,
    plot_title: &str,
    bounds: ((f64, f64), (f64, f64)),
) {
    println!("Generating {}", plot_title);

    let mut line_group = Vec::new();
    let mut line = Vec::new();

    let mut curr_dist = paths[0].dist;
    for path in paths {
        if path.dist != curr_dist {
            line_group.push(line);
            line = Vec::new();
            curr_dist = path.dist;
        }
        let val = property(path);
        line.push(val);
    }
    plot_with_title(
        &format!("{}", plot_title),
        &format!(
            "generate_artifacts/output/approximation_function/{}/{}.png",
            folder, file_name
        ),
        &line_group,
        bounds,
    )
    .unwrap();
}
