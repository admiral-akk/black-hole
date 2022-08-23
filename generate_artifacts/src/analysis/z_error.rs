use std::{f64::consts::TAU, fs};

use test_utils::plot_with_title;

use super::angle_test_point::AngleTestPoint;

fn normalize_line_z(line: &mut Vec<(f32, f32)>) {
    if line.len() < 2 {
        return;
    }
    let (min, max) = (
        line.iter()
            .map(|v| v.0)
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap(),
        line.iter()
            .map(|v| v.0)
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap(),
    );
    for i in 0..line.len() {
        line[i].0 = (line[i].0 - min) / (max - min);
    }
}

fn log_error(line: &mut Vec<(f32, f32)>) {
    line.iter_mut()
        .for_each(|point| point.1 = (point.1 + 1.).log2());
}

fn plot_error(cache: &AngleDistanceCache, mut plots: Vec<Vec<Vec<(f32, f32)>>>, dists: Vec<f64>) {
    let folder_path = &format!(
        "generate_artifacts/output/distance_cache_{}_{}_{}/z_error",
        cache.params.dist.size, cache.params.view_dist.size, cache.params.angle.size
    );
    fs::create_dir_all(folder_path).unwrap();
    for (i, lines) in plots.iter_mut().enumerate() {
        lines.iter_mut().for_each(|_line| {});
        plot_with_title(
            &format!("Error for dist = {:.2}", dists[i]),
            &format!("{}/dist_{:.2}.png", folder_path, dists[i]),
            &lines,
            ((0., TAU), (0., 5.)),
        )
        .unwrap();
        lines.iter_mut().for_each(|line| log_error(line));
        plot_with_title(
            &format!("Error for dist = {:.2}", dists[i]),
            &format!("{}/log_error_dist_{:.2}.png", folder_path, dists[i]),
            &lines,
            ((0., TAU), (0., 5.)),
        )
        .unwrap();
    }
}

pub fn plot_angle_error_by_z(
    cache: &AngleDistanceCache,
    results: &Vec<(AngleTestPoint, Option<f32>)>,
) {
    println!("Generating z error plots");
    let mut filtered: Vec<&(AngleTestPoint, Option<f32>)> =
        results.iter().filter(|_p| true).collect();
    filtered.sort_by(|p_1, p_2| {
        let order = p_1.0.dist.partial_cmp(&p_2.0.dist).unwrap();
        if order.is_eq() {
            let order = p_1
                .0
                .view_port_coord
                .partial_cmp(&p_2.0.view_port_coord)
                .unwrap();
            if order.is_eq() {
                return p_1.0.target_angle.partial_cmp(&p_2.0.target_angle).unwrap();
            } else {
                return order;
            }
        } else {
            return order;
        }
    });

    let folder_path = &format!(
        "generate_artifacts/output/distance_cache_{}_{}_{}/z_error",
        cache.params.dist.size, cache.params.view_dist.size, cache.params.angle.size
    );
    fs::create_dir_all(folder_path).unwrap();
    let mut curr_view = results[0].0.view_port_coord;
    let mut curr_dist = results[0].0.dist;
    let mut plots = Vec::new();
    let mut dists = Vec::new();
    let mut lines = Vec::new();
    let mut line = Vec::new();
    for point in filtered {
        if point.0.dist != curr_dist {
            lines.push(line);
            plots.push(lines);
            dists.push(curr_dist);
            curr_dist = point.0.dist;
            curr_view = point.0.target_angle;
            lines = Vec::new();
            line = Vec::new();
        }
        if point.0.view_port_coord != curr_view {
            lines.push(line);
            curr_dist = point.0.dist;
            curr_view = point.0.view_port_coord;
            line = Vec::new();
        }
        if point.0.dist_at_angle.is_none() {
            continue;
        }

        let val = point.0.dist_at_angle.unwrap();
        if val < 2. || val > 12. {
            continue;
        }
        if point.1.is_none() {
            continue;
        }
        let approx = point.1.unwrap();
        if 2. > approx || approx > 12. {
            continue;
        }
        let diff = (point.1.unwrap() - val as f32).abs();

        line.push((point.0.target_angle as f32, diff));
    }
    dists.push(curr_dist);
    lines.push(line);
    plots.push(lines);
    plot_error(&cache, plots, dists);
}
