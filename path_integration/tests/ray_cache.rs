#[cfg(test)]

mod tests {
    use path_integration::cache::ray_cache::RayCache;
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
            let path = &lines[i];
            let r = (255.0 * i as f32 / (lines.len() - 1) as f32) as u8;
            let color = RGBColor(r, 255 - r, 0);
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
    fn plot_cached_final_dir() -> Result<(), Box<dyn std::error::Error>> {
        let distance = 5.0;
        let r = 1.0;
        let cache_size = 512;
        let ray_cache = RayCache::compute_new(cache_size, r, distance);
        let mut lines: Vec<Vec<(f32, f32)>> = Vec::new();
        let mut line: Vec<(f32, f32)> = Vec::new();
        for i in 0..ray_cache.cache.len() {
            let answer = &ray_cache.cache[i];
            line.push((answer.z, answer.final_dir[2]));
        }
        lines.push(line);
        let path = format!("output/ray_cache/z_to_z.png");
        plot_trajectories(&path, &lines, ((-1., 1.), (-1., 1.)))?;
        Ok(())
    }
}
