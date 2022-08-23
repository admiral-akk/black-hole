use std::fs;

use test_utils::plot_with_title;
use wire_structs::gpu::gpu_state::SimulatedRay;

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

pub fn plot_test_statistics(
    folder_path: &String,
    rays: &Vec<SimulatedRay>,
    dists: &Vec<f32>,
    views: &Vec<f32>,
    angles: &Vec<f32>,
) {
    fs::create_dir_all(folder_path).unwrap();
    let mut path_plots = Vec::new();

    println!("Generating cache stats");
    for (d_index, dist) in dists.iter().enumerate() {
        let rays = &rays[d_index * views.len()..(d_index + 1) * views.len()];
        let mut path_plot = Vec::new();
        for (v_index, _view) in views.iter().enumerate() {
            let mut path = Vec::new();
            for (a_index, angle) in angles.iter().enumerate() {
                let val = rays[v_index].angle_dist[a_index];
                if val == 0. {
                    break;
                }
                let (x, y) = (angle.sin() * val, -angle.cos() * val);
                path.push((x, y));
            }
            path_plot.push(path);
        }
        path_plots.push((path_plot, *dist));
    }
    plot_paths(folder_path, path_plots);
}
