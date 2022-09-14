use std::{
    f32::consts::{FRAC_PI_2, PI},
    f64::consts::TAU,
};

use test_utils::plot_with_title;
use wire_structs::sampler::{dimension_params::DimensionParams, simulated_path::SimulatedPath};

pub fn analyze_paths(paths: &Vec<SimulatedPath>, angle: &DimensionParams) {
    let angles = angle.generate_list();
    plot_property_by_path(
        paths,
        &|path| (path.view, path.final_angle(&angles)),
        "combined_dir",
        "combined_dir",
        "Final Direction or Angle if escaped",
        ((0., 1.), (0., TAU + FRAC_PI_2 as f64)),
    );
    plot_path(paths, angle);
    plot_property_by_path(
        paths,
        &|path| {
            let min_angle = match path
                .ray
                .angle_dist
                .iter()
                .enumerate()
                .filter(|(_, d)| **d > 0.)
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            {
                Some((i, _)) => angles[i],
                None => angles[0],
            };
            (path.view, path.projected_min_angle(&angles) - min_angle)
        },
        "mid_dir",
        "mid_dir",
        "Mid Angle",
        ((0., 1.), (-PI as f64, PI as f64)),
    );
    plot_property_by_path(
        paths,
        &|path| {
            let final_index = path
                .ray
                .angle_dist
                .iter()
                .enumerate()
                .filter(|(_i, dist)| **dist > 0.)
                .last();
            let final_index = match final_index {
                Some((i, _)) => i,
                None => 0,
            };
            (path.view, angles[final_index])
        },
        "final_angle",
        "final_angle",
        "Final Angle",
        ((0., 1.), (0., TAU)),
    );
    plot_property_by_path(
        paths,
        &|path| {
            let final_dir = path.ray.final_dir;
            let final_angle = (f32::atan2(final_dir[1], final_dir[0]) + TAU as f32) % TAU as f32;
            (path.view, final_angle)
        },
        "final_dir",
        "final_dir",
        "Final Direction",
        ((0., 1.), (0., TAU)),
    );
    plot_property_by_path(
        paths,
        &|path| {
            let min = match path
                .ray
                .angle_dist
                .iter()
                .filter(|d| **d > 0.)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
            {
                Some(v) => *v,
                None => 1.5,
            };
            (path.view, min)
        },
        "closest_point",
        "closest_point",
        "Closest Dist",
        ((0., 1.), (0., 20.)),
    );
    plot_property_by_path(
        paths,
        &|path| {
            let min = match path
                .ray
                .angle_dist
                .iter()
                .enumerate()
                .filter(|(_, d)| **d > 0.)
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            {
                Some((i, _)) => i,
                None => 0,
            };
            let min_dist = match path
                .ray
                .angle_dist
                .iter()
                .filter(|d| **d > 0.)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
            {
                Some(v) => *v,
                None => 1.5,
            };
            let angle;
            if min_dist <= 2. {
                angle = angles[min];
            } else {
                angle = TAU as f32 - angles[min];
            }
            (path.view, angle)
        },
        "angle_closest_point",
        "angle_closest_point",
        "Angle at Closest Dist",
        ((0., 1.), (0., TAU)),
    );
    plot_property_by_step(
        paths,
        &|path, angles| {
            let mut path_vec = Vec::new();
            for (i, dist) in path.ray.angle_dist.iter().enumerate() {
                if *dist == 0. {
                    break;
                }
                path_vec.push((angles[i], *dist));
            }
            path_vec
        },
        "distance",
        "distance",
        "Distance by Angle",
        ((0., TAU), (0., 30.)),
        angle,
    );
}

fn plot_path(paths: &Vec<SimulatedPath>, angle: &DimensionParams) {
    println!("Generating {}", "paths");

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
                "generate_artifacts/output/paths/{}/{}_{}.png",
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
    property: &dyn Fn(&SimulatedPath, &Vec<f32>) -> Vec<(f32, f32)>,
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
    for path in paths {
        let line = property(path, &angles);
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
                "generate_artifacts/output/paths/{}/{}_{}.png",
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
            "generate_artifacts/output/paths/{}/{}.png",
            folder, file_name
        ),
        &line_group,
        bounds,
    )
    .unwrap();
}
