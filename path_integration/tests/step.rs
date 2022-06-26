#[cfg(test)]

mod tests {
    use glam::DVec3;
    use path_integration::{cast_ray_steps, cast_ray_steps_debug, Field, Ray};
    use plotters::prelude::*;
    use plotters::{
        prelude::{BitMapBackend, ChartBuilder, Circle, EmptyElement, IntoDrawingArea, LineSeries},
        style::{IntoFont, BLACK, WHITE},
    };

    fn trim_path(x_min: f64, x_max: f64, y_min: f64, y_max: f64, path: &Vec<DVec3>) -> Vec<DVec3> {
        let mut trim_path = Vec::new();
        for i in 0..path.len() {
            let (x, y) = (path[i].x, path[i].y);
            if x < x_min || x > x_max || y < y_min || y > y_max {
                let (prev_x, prev_y) = (path[i - 1].x, path[i - 1].y);
                let (final_x, final_y) = (path[i].x, path[i].y);
                let (slope_x, slope_y) = (final_x - prev_x, final_y - prev_y);
                let mut t = 1.0;
                if final_x < x_min {
                    t = f64::min((x_min - prev_x) / slope_x, t);
                }
                if final_x > x_max {
                    t = f64::min((x_max - prev_x) / slope_x, t);
                }
                if final_y < y_min {
                    t = f64::min((y_min - prev_y) / slope_y, t);
                }
                if final_y > y_max {
                    t = f64::min((y_max - prev_y) / slope_y, t);
                }
                trim_path.pop();
                trim_path.push(DVec3::new(prev_x + t * slope_x, prev_y + t * slope_y, 0.0));
                break;
            } else {
                trim_path.push(path[i]);
            }
        }
        trim_path
    }

    fn plot_trajectories(
        file_path: &str,
        field: &Field,
        lines: &Vec<Vec<DVec3>>,
        is_hit: &Vec<bool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&file_path, (2000, 2000)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("schwarzchild radius={}", field.schwarzchild_radius()),
                ("Arial", 50).into_font(),
            )
            .margin(5 as u32)
            .x_label_area_size(30 as u32)
            .y_label_area_size(30 as u32)
            .build_cartesian_2d(0.0f64..10.0f64, 0.0f64..10.0f64)?;

        chart.configure_mesh().draw()?;

        let _start = DVec3::new(5.0, 0.0, 0.0);

        chart.draw_series(PointSeries::of_element(
            vec![(field.center.x as f64, field.center.y as f64)],
            5,
            &BLACK,
            &|c, s: u32, st| {
                return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
            },
        ))?;

        for i in 0..lines.len() {
            let r = (i as f64) / ((lines.len() as f64) - 1.0);
            let path = &lines[i];
            let trim = trim_path(0.0, 10.0, 0.0, 10.0, path);
            let mut color = RGBColor((255.0 * r) as u8, (255.0 * (1.0 - r)) as u8, 0);
            if is_hit[i] {
                color = BLACK;
            }
            chart.draw_series(LineSeries::new(
                trim.iter().map(|v| (v.x as f64, v.y as f64)),
                &color,
            ))?;
        }
        // And if we want SVG backend
        // let backend = SVGBackend::new("output.svg", (800, 600));
        root.present().expect("Plot failed!");
        Ok(())
    }

    fn find_near_miss(field: &Field, max_width: f64) -> (DVec3, DVec3) {
        let start = DVec3::new(5.0, 0.0, 0.0);
        let mut left = DVec3::new(0.0, 10.0, 0.0);
        let mut right = DVec3::new(5.0, 10.0, 0.0);
        while (left - right).length() > max_width {
            let center = 0.5 * (left + right);
            let ray = Ray::new(start, (center - start).normalize());
            if cast_ray_steps(&ray, field, 100.0).is_none() {
                // hit the black hole
                right = center;
            } else {
                left = center;
            }
        }
        (left, right)
    }

    #[test]
    fn plot_fixed_radius() -> Result<(), Box<dyn std::error::Error>> {
        let start = DVec3::new(5.0, 0.0, 0.0);
        let start2 = 5.0 * DVec3::X;
        for radius in 1..=10 {
            let r = (radius as f64) / 10.0;
            let field = Field::new(DVec3::new(5.0, 5.0, 0.0), r, &start2);
            let mut lines: Vec<Vec<DVec3>> = Vec::new();
            let num_lines = 1000;
            let mut is_hit: Vec<bool> = Vec::new();
            for i in 0..num_lines {
                let r = (i as f64) / ((num_lines as f64) - 1.0);
                let end = DVec3::new(10.0 * r, 10.0, 0.0);
                let ray = Ray::new(start, end - start);
                is_hit.push(cast_ray_steps(&ray, &field, 100.0).is_none());
                let path = cast_ray_steps_debug(&ray, &field, 40.0);
                lines.push(path);
            }
            let path = format!("output/radius/test_rad_{}_paths.png", r);
            plot_trajectories(&path, &field, &lines, &is_hit)?;
        }
        Ok(())
    }

    #[test]
    fn plot_fixed_radius_near_paths() -> Result<(), Box<dyn std::error::Error>> {
        let start = DVec3::new(5.0, 0.0, 0.0);
        let start2 = 5.0 * DVec3::X;
        for radius in 1..=10 {
            let r = (radius as f64) / 10.0;
            let field = Field::new(DVec3::new(5.0, 5.0, 0.0), r, &start2);
            let (left, _) = find_near_miss(&field, 0.000001);
            let mut lines: Vec<Vec<DVec3>> = Vec::new();
            let num_lines = 1000;
            let mut is_hit: Vec<bool> = Vec::new();

            for i in 0..num_lines {
                let r = (i as f64) / ((num_lines as f64) - 1.0);
                let end = left - 0.1 * (1.0 - r) * DVec3::X;
                let ray = Ray::new(start, end - start);
                is_hit.push(cast_ray_steps(&ray, &field, 100.0).is_none());
                let path = cast_ray_steps_debug(&ray, &field, 40.0);
                lines.push(path);
            }
            let path = format!("output/near/test_rad_{}_paths.png", r);
            plot_trajectories(&path, &field, &lines, &is_hit)?;
        }
        Ok(())
    }
}
