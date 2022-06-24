#[cfg(test)]

mod tests {
    use geometry::{Ray, Vec3};
    use path_integration::{cast_ray_steps, cast_ray_steps_debug};
    use plotters::prelude::*;
    use plotters::{
        prelude::{
            BitMapBackend, ChartBuilder, Circle, DrawingBackend, EmptyElement, IntoDrawingArea,
            LineSeries, PathElement,
        },
        style::{Color, IntoFont, ShapeStyle, BLACK, GREEN, RED, WHITE},
    };

    fn plot_trajectories(
        field_scale: f32,
        field_center: &Vec3,
        lines: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("output/test_{}_paths.png", field_scale);
        let mut root = BitMapBackend::new(&path, (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption(format!("f={}", field_scale), ("Arial", 50).into_font())
            .margin(5 as u32)
            .x_label_area_size(30 as u32)
            .y_label_area_size(30 as u32)
            .build_cartesian_2d(0.0f32..10.0f32, 0.0f32..10.0f32)?;

        chart.configure_mesh().draw()?;

        let start = Vec3::new(5.0, 0.0, 0.0);

        chart.draw_series(PointSeries::of_element(
            vec![(field_center.x, field_center.y)],
            5,
            &BLACK,
            &|c, s, st| {
                return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
            },
        ))?;

        for x in 0..lines {
            let r = (x as f32) / ((lines as f32) - 1.0);

            let end = Vec3::new(10.0 * r, 10.0, 0.0);
            let ray = Ray::new(start, end - start);
            let steps = cast_ray_steps_debug(&ray, field_scale, &field_center);

            let mut debug = Vec::new();
            for i in 0..steps.len() {
                let (x, y) = (steps[i].x, steps[i].y);
                if x < 0.0 || x > 10.0 || y < 0.0 || y > 10.0 {
                    let (prev_x, prev_y) = (steps[i - 1].x, steps[i - 1].y);
                    let (final_x, final_y) = (x, y);
                    let (slope_x, slope_y) = (final_x - prev_x, final_y - prev_y);
                    let mut t = 1.0;
                    if final_x < 0.0 {
                        t = f32::min(-prev_x / slope_x, t);
                    }
                    if final_x > 10.0 {
                        t = f32::min((10.0 - prev_x) / slope_x, t);
                    }
                    if final_y < 0.0 {
                        t = f32::min(-prev_y / slope_y, t);
                    }
                    if final_y > 10.0 {
                        t = f32::min((10.0 - prev_y) / slope_y, t);
                    }
                    debug.push((prev_x + slope_x * t, prev_y + slope_y * t));
                    break;
                }
                debug.push((x, y));
            }
            let mut color = RGBColor((255.0 * r) as u8, (255.0 * (1.0 - r)) as u8, 0);
            if cast_ray_steps(&ray, field_scale, field_center).is_none() {
                color = BLACK;
            }

            chart.draw_series(LineSeries::new(debug, &color))?;
        }
        // And if we want SVG backend
        // let backend = SVGBackend::new("output.svg", (800, 600));
        root.present().expect("Plot failed!");
        Ok(())
    }

    fn find_near_miss(field_scale: f32, field_center: &Vec3) -> (Vec3, Vec3) {
        let start = Vec3::new(5.0, 0.0, 0.0);
        let mut left = Vec3::new(0.0, 10.0, 0.0);
        let mut right = *field_center;
        while (left - right).length() > 0.0001 {
            let center = 0.5 * (left + right);
            let ray = Ray::new(start, center.normalize());
            let steps = cast_ray_steps_debug(&ray, field_scale, &field_center);
            if steps.len() < 10000 {
                // hit the black hole
                right = center;
            } else {
                left = center;
            }
        }
        (left, right)
    }

    #[test]
    fn plot_all_trajectories() -> Result<(), Box<dyn std::error::Error>> {
        for scale in 0..=10 {
            plot_trajectories((scale as f32) / 10.0, &Vec3::new(5.0, 5.0, 0.0), 100)?;
        }
        Ok(())
    }

    #[test]
    fn plot_near_trajectory() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
