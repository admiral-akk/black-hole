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

    fn trim_path(x_min: f32, x_max: f32, y_min: f32, y_max: f32, path: &Vec<Vec3>) -> Vec<Vec3> {
        let mut trim_path = Vec::new();
        for i in 0..path.len() {
            let (x, y) = (path[i].x, path[i].y);
            if x < x_min || x > x_max || y < y_min || y > y_max {
                let (prev_x, prev_y) = (path[i - 1].x, path[i - 1].y);
                let (final_x, final_y) = (path[i].x, path[i].y);
                let (slope_x, slope_y) = (final_x - prev_x, final_y - prev_y);
                let mut t = 1.0;
                if final_x < x_min {
                    t = f32::min((x_min - prev_x) / slope_x, t);
                }
                if final_x > x_max {
                    t = f32::min((x_max - prev_x) / slope_x, t);
                }
                if final_y < y_min {
                    t = f32::min((y_min - prev_y) / slope_y, t);
                }
                if final_y > y_max {
                    t = f32::min((y_max - prev_y) / slope_y, t);
                }
                trim_path.pop();
                trim_path.push(Vec3::new(prev_x + t * slope_x, prev_y + t * slope_y, 0.0));
                break;
            } else {
                trim_path.push(path[i]);
            }
        }
        trim_path
    }

    fn plot_trajectories(
        field_scale: f32,
        field_center: &Vec3,
        lines: &Vec<Vec<Vec3>>,
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

        for i in 0..lines.len() {
            let r = (i as f32) / ((lines.len() as f32) - 1.0);
            let path = &lines[i];
            let trim = trim_path(0.0, 10.0, 0.0, 10.0, path);
            let mut color = RGBColor((255.0 * r) as u8, (255.0 * (1.0 - r)) as u8, 0);
            chart.draw_series(LineSeries::new(trim.iter().map(|v| (v.x, v.y)), &color))?;
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
        let start = Vec3::new(5.0, 0.0, 0.0);
        for scale in 0..=10 {
            let (field_scale, field_center) = ((scale as f32) / 10.0, &Vec3::new(5.0, 5.0, 0.0));
            let mut lines: Vec<Vec<Vec3>> = Vec::new();
            for i in 0..100 {
                let r = (i as f32) / 99.0;
                let end = Vec3::new(10.0 * r, 10.0, 0.0);
                let ray = Ray::new(start, end - start);
                let path = cast_ray_steps_debug(&ray, field_scale, &field_center);
                lines.push(path);
            }
            plot_trajectories((scale as f32) / 10.0, &Vec3::new(5.0, 5.0, 0.0), &lines)?;
        }
        Ok(())
    }

    #[test]
    fn plot_near_trajectory() -> Result<(), Box<dyn std::error::Error>> {
        let field_center = Vec3::new(5.0, 5.0, 0.0);
        let (left, right) = find_near_miss(0.1, &field_center);
        Ok(())
    }
}
