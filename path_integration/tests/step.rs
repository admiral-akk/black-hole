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

    #[test]
    fn plot_straight_trajectories() -> Result<(), Box<dyn std::error::Error>> {
        let field_scale = 0.0;
        let field_center = Vec3::new(5.0, 5.0, 0.0);

        let mut root =
            BitMapBackend::new("output/test_straight_paths.png", (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption("y=x^2", ("Arial", 50).into_font())
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

        for x in 0..=10 {
            let end = Vec3::new(x as f32, 10.0, 0.0);
            let ray = Ray::new(start, end - start);
            let steps = cast_ray_steps_debug(&ray, field_scale, &field_center);

            let mut debug = Vec::new();
            for i in 0..steps.len() {
                let (x, y) = (steps[i].x, steps[i].y);
                if x < 0.0 || x > 10.0 || y < 0.0 || y > 10.0 {
                    break;
                }
                debug.push((x, y));
            }
            chart.draw_series(LineSeries::new(debug, &RED))?;
        }
        // And if we want SVG backend
        // let backend = SVGBackend::new("output.svg", (800, 600));
        root.present().expect("Plot failed!");
        Ok(())
    }
}
