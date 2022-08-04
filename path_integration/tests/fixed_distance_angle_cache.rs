#[cfg(test)]

mod tests {
    use std::fs;

    use path_integration::cache::angle_cache::{AngleCache, TestStats};
    use path_integration::cast_ray_steps_response;
    use plotters::prelude::*;
    use plotters::{
        prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea, LineSeries},
        style::{IntoFont, WHITE},
    };

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
            let r = (255.0 * i as f32 / (lines.len() - 1) as f32) as u8;
            let mut color = RGBColor(r, 255 - r, 0);
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
    fn print_angle_error() {
        let black_hole_radius = 1.5;
        let distance = (7.0, 20.0);
        let cache =
            serde_json::from_str::<AngleCache>(&fs::read_to_string("angle_cache.txt").unwrap())
                .unwrap();
        // AngleCache::compute_new(cache_size, black_hole_radius, distance, max_disc_radius);

        let iterations = 20;
        let mut lines = Vec::new();
        for d in 0..=iterations {
            let dist = (d as f32 / iterations as f32) * (distance.1 - distance.0) + distance.0;
            let mut line = Vec::new();
            for angle in 1..80 {
                let angle = std::f64::consts::TAU * angle as f64 / 80.0;
                let mut test_stats = TestStats::default();
                for x in 0..=iterations {
                    let x = (x as f64) / (iterations as f64);
                    let z = (1. - x * x).sqrt();
                    println!("dist: {}, angle: {}, z: {}", dist, angle, z);
                    if z > 0.9999 {
                        continue;
                    }
                    let path = cast_ray_steps_response(z, dist as f64, black_hole_radius as f64)
                        .get_angle_dist();

                    let approx_dist = cache.get_dist(dist, z as f32, angle as f32);
                    if path.get_max_angle() < angle {
                        continue;
                    }
                    let true_dist = path.get_dist(angle);
                    assert!(
                        true_dist.is_some(),
                        "Angle is missing. Angle: {}, z: {}",
                        angle,
                        z
                    );
                    let true_dist = true_dist.unwrap() as f32;
                    if approx_dist.is_none() {
                    } else {
                        test_stats.add_sample(
                            z as f32,
                            dist,
                            angle as f32,
                            true_dist,
                            approx_dist.unwrap(),
                        );
                    }
                }

                if test_stats.max_error > 0. {
                    line.push((angle as f32, test_stats.max_error));
                }
            }
            lines.push(line);
        }
        let path = "output/angle_cache/angle_errors.png";
        plot_trajectories(
            &path,
            &lines,
            ((0. as f64, std::f64::consts::TAU), (0.0, 1.0)),
        )
        .unwrap();
    }
}
