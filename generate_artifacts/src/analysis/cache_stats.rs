use std::{f64::consts::TAU, fs};

use test_utils::plot_with_title;
use wire_structs::angle_distance_cache::AngleDistanceCache;

fn plot_distances(folder_path: &String, distance_plots: Vec<(Vec<Vec<(f32, f32)>>, f32)>) {
    for (plot, dist) in distance_plots {
        plot_with_title(
            &format!("Distance by angle, dist = {:.2}", dist),
            &format!("{}/dist_per_angle_{:.2}.png", folder_path, dist),
            &plot,
            ((0., 1.), (0., 35.)),
        )
        .unwrap();
    }
}

fn plot_paths(folder_path: &String, path_plots: Vec<(Vec<Vec<(f32, f32)>>, f32)>) {
    for (plot, dist) in path_plots {
        plot_with_title(
            &format!("Distance by angle, dist = {:.2}", dist),
            &format!("{}/path_dist_{:.2}.png", folder_path, dist),
            &plot,
            ((-30., 30.), (-30., 30.)),
        )
        .unwrap();
    }
}

pub fn plot_cache_statistics(cache: &AngleDistanceCache) {
    let disc_bounds = [2., 12.];

    let params = &cache.params;
    let dists = params.dist.generate_list();
    let angles = params.angle.generate_list();
    let views = params.view_dist.generate_list();

    let folder_path = &format!(
        "generate_artifacts/output/distance_cache_{}_{}_{}/cache_stats",
        cache.params.dist.size, cache.params.view_dist.size, cache.params.angle.size
    );
    fs::create_dir_all(folder_path).unwrap();
    let mut data_points_per = Vec::new();
    let mut distance_plots = Vec::new();
    let mut path_plots = Vec::new();

    println!("Generating cache stats");
    for (d_index, dist) in dists.iter().enumerate() {
        let mut distance_plot = Vec::new();
        let mut path_plot = Vec::new();
        let mut data_per = Vec::new();
        for (v_index, _view) in views.iter().enumerate() {
            let mut path = Vec::new();
            for (a_index, angle) in angles.iter().enumerate() {
                let val = cache.distances[d_index][v_index][a_index];
                let (x, y) = (angle.sin() * val, -angle.cos() * val);
                path.push((x, y));
            }
            path_plot.push(path);
        }
        path_plots.push((path_plot, *dist));
        for (a_index, angle) in angles.iter().enumerate() {
            let mut distance_for_angle = Vec::new();
            let mut count = 0;
            for (v_index, view) in views.iter().enumerate() {
                let val = cache.distances[d_index][v_index][a_index];
                distance_for_angle.push((*view, val));
                if val >= disc_bounds[0] && val <= disc_bounds[1] {
                    count += 1;
                }
            }
            distance_plot.push(distance_for_angle);
            data_per.push((*angle, count as f32 / (10.) as f32));
        }
        distance_plots.push((distance_plot, *dist));
        data_points_per.push(data_per);
    }
    plot_distances(folder_path, distance_plots);
    plot_paths(folder_path, path_plots);
    plot_with_title(
        &format!("Data points per angle"),
        &format!("{}/points_per_angle.png", folder_path),
        &data_points_per,
        ((0., TAU), (0., 1.)),
    )
    .unwrap();
}
