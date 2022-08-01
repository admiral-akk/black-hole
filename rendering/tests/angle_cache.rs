#[cfg(test)]

mod tests {
    use plotters::prelude::*;
    use plotters::{
        prelude::{BitMapBackend, ChartBuilder, Circle, EmptyElement, IntoDrawingArea, LineSeries},
        style::{IntoFont, BLACK, WHITE},
    };
    use rendering::structs::angle_cache::AngleCache;

    fn plot_trajectories(
        file_path: &str,
        lines: &Vec<Vec<(f32, f32)>>,
        bounds: ((f64, f64), (f64, f64)),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&file_path, (2000, 2000)).into_drawing_area();
        root.fill(&WHITE)?;

        let (x_bound, y_bound) = bounds;
        let mut chart = ChartBuilder::on(&root)
            .caption(format!("angle cache"), ("Arial", 50).into_font())
            .margin(5 as u32)
            .x_label_area_size(30 as u32)
            .y_label_area_size(30 as u32)
            .build_cartesian_2d((x_bound.0)..(x_bound.1), (y_bound.0)..(y_bound.1))?;

        chart.configure_mesh().draw()?;
        for i in 0..lines.len() {
            let r = (i as f64) / ((lines.len() as f64) - 1.0);
            let path = &lines[i];
            let mut color = RGBColor(255, 0, 0);
            chart.draw_series(LineSeries::new(
                path.iter().map(|v| (v.0 as f64, v.1 as f64)),
                &color,
            ))?;
        }
        // And if we want SVG backend
        // let backend = SVGBackend::new("output.svg", (800, 600));
        root.present().expect("Plot failed!");
        Ok(())
    }

    #[test]
    fn plot_first_hit_angle_cache() -> Result<(), Box<dyn std::error::Error>> {
        let cache_size = 30;
        let black_hole_radius = 1.5;
        let camera_distance = 10.0;
        let disc_radius = 3.0;
        let angle_cache =
            AngleCache::compute_new(cache_size, black_hole_radius, camera_distance, disc_radius);
        let mut lines: Vec<Vec<(f32, f32)>> = Vec::new();
        let mut line: Vec<(f32, f32)> = Vec::new();

        for x in 0..angle_cache.cache.len() {
            let z = angle_cache.cache[x].z;
            line.push((z, angle_cache.cache[x].angle));
            continue;
            let angle_dist = &angle_cache.cache[x].angle_dist;
            for i in 0..angle_dist.len() {
                let curr = angle_dist[i];
                if curr.1 <= disc_radius {
                    break;
                    let prev = angle_dist[i - 1];
                    let prev_weight = (prev.1 - disc_radius) / (prev.1 - curr.1);

                    line.push((z, prev_weight * prev.0 + (1.0 - prev_weight) * curr.0));
                    break;
                }
            }
        }
        lines.push(line);
        let path = format!("output/angle_cache/test_first_hit.png");
        plot_trajectories(
            &path,
            &lines,
            ((0.9 as f64, 1.0), (0.0, std::f64::consts::TAU)),
        )?;
        Ok(())
    }
}
