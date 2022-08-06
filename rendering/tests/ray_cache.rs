#[cfg(test)]

mod tests {
    use glam::{Vec3};
    use image::Rgba;
    
    use plotters::prelude::*;
    use plotters::{
        prelude::{BitMapBackend, ChartBuilder, Circle, EmptyElement, IntoDrawingArea, LineSeries},
        style::{IntoFont, BLACK, WHITE},
    };
    use rendering::structs::ray_cache::RayCache;

    fn trim_path(
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        path: &Vec<(f32, f32)>,
    ) -> Vec<(f32, f32)> {
        let mut trim_path = Vec::new();
        let mut i = 0;
        while i < path.len() {
            let (x, y) = (path[i].0, path[i].1);
            while x < x_min || x > x_max || y < y_min || y > y_max {
                i += 1;
            }
        }

        for i in 0..path.len() {
            let (x, y) = (path[i].0, path[i].1);
            if x < x_min || x > x_max || y < y_min || y > y_max {
                continue;
            }
            if x < x_min || x > x_max || y < y_min || y > y_max {
                let (prev_x, prev_y) = (path[i - 1].0, path[i - 1].1);
                let (final_x, final_y) = (path[i].0, path[i].1);
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
                trim_path.push((prev_x + t * slope_x, prev_y + t * slope_y));
                break;
            } else {
                trim_path.push(path[i]);
            }
        }
        trim_path
    }

    fn plot_trajectories(
        file_path: &str,
        lines: &Vec<Vec<(f32, f32)>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&file_path, (2000, 2000)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption(format!("angle cache"), ("Arial", 50).into_font())
            .margin(5 as u32)
            .x_label_area_size(30 as u32)
            .y_label_area_size(30 as u32)
            .build_cartesian_2d(0.0f64..std::f64::consts::TAU, 0.0f64..10.0f64)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(PointSeries::of_element(
            vec![(0.0, 0.0)],
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
            let color = RGBColor((255.0 * r) as u8, (255.0 * (1.0 - r)) as u8, 0);
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
    fn plot_angle_ray_cache() -> Result<(), Box<dyn std::error::Error>> {
        let ray_cache = RayCache::compute_new(1024, 1.5, 17.0);
        let mut lines: Vec<Vec<(f32, f32)>> = Vec::new();
        for x in 0..ray_cache.angle_cache.len() {
            let mut next_line = Vec::new();
            let pr = ray_cache.angle_cache[x]
                .angle_to_dist
                .iter()
                .find(|x| x.1 <= 3.);
            if pr.is_none() {
                continue;
            }
            for y in 0..ray_cache.angle_cache[x].angle_to_dist.len() {
                next_line.push(ray_cache.angle_cache[x].angle_to_dist[y]);
            }
            lines.push(next_line);
        }
        let path = format!("output/ray_cache/test_angle_paths.png");
        plot_trajectories(&path, &lines)?;
        Ok(())
    }

    fn float_to_u16(v: f32) -> u16 {
        let v = std::u16::MAX as f32 * ((v + 1.0) / 2.0);
        if v > std::u16::MAX as f32 {
            return std::u16::MAX;
        }
        if v <= 0.0 {
            return 0;
        }
        return v as u16;
    }
    fn u16_to_float(v: u16) -> f32 {
        (2.0 * v as f32 / std::u16::MAX as f32) - 1.0
    }

    #[test]
    fn write_and_read_cache_from_image() -> Result<(), Box<dyn std::error::Error>> {
        let (x_dim, y_dim) = (512, 64);
        let mut float_buffer: image::ImageBuffer<image::Rgba<u16>, Vec<u16>> =
            image::ImageBuffer::new(x_dim, y_dim);
        let mut z_max_buffer: image::ImageBuffer<image::Rgba<u16>, Vec<u16>> =
            image::ImageBuffer::new(y_dim, 1);
        let (min, max) = (5., 20.);
        for y in 0..y_dim {
            let dist = (max - min) * (y as f32) / (y_dim - 1) as f32 + min;
            let cache = RayCache::compute_new(x_dim as usize, 1.5, dist);
            z_max_buffer.put_pixel(
                y as u32,
                0 as u32,
                Rgba([float_to_u16(cache.max_z), 0, 0, u16::MAX]),
            );
            for x in 0..cache.cache.len() {
                let final_dir = cache.cache[x].final_dir;
                float_buffer.put_pixel(
                    x as u32,
                    y as u32,
                    Rgba([
                        float_to_u16(final_dir.x),
                        float_to_u16(final_dir.y),
                        float_to_u16(final_dir.z),
                        u16::MAX,
                    ]),
                )
            }
        }

        let path = "output/ray_cache/cache.png";
        let z_path = "output/ray_cache/z_max_cache.png";
        float_buffer.save(path)?;
        z_max_buffer.save(z_path)?;
        let img = image::io::Reader::open(path)?.decode()?;
        let z_img = image::io::Reader::open(z_path)?.decode()?;
        let read_buffer = img.as_rgba16().unwrap();
        let z_read_buffer = z_img.as_rgba16().unwrap();

        for y in 0..y_dim {
            let dist = (max - min) * (y as f32) / (y_dim - 1) as f32 + min;
            let cache = RayCache::compute_new(x_dim as usize, 1.5, dist);
            let image_dir = z_read_buffer
                .get_pixel(y as u32, 0)
                .0
                .map(|v| u16_to_float(v));
            println!("max_z:{}", image_dir[0]);
            assert!(
                f32::abs(cache.max_z - image_dir[0]) < 0.0001,
                "expected: {:?}\nactual: {:?}\n",
                cache.max_z,
                image_dir[0]
            );
            for x in 0..cache.cache.len() {
                let final_dir = cache.cache[x].final_dir;
                let image_dir = read_buffer
                    .get_pixel(x as u32, y)
                    .0
                    .map(|v| u16_to_float(v));
                let image_final_dir = Vec3::from_slice(&image_dir[0..3]);
                assert!(
                    (image_final_dir - final_dir).length() < 0.01,
                    "expected: {:?}\nactual: {:?}\n",
                    final_dir,
                    image_final_dir
                );
            }
        }
        Ok(())
    }
}
