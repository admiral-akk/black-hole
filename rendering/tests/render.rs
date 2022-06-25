#[cfg(test)]

mod tests {
    use std::path::Path;

    use geometry::DVec3;
    use rendering::{
        render::render,
        structs::{black_hole::BlackHole, camera::Camera, dimensions::Dimensions, stars::Stars},
    };

    fn write_image(image_name: &str, buffer: &[u8], dimensions: &Dimensions) {
        image::save_buffer(
            &Path::new(&format!("output/{}.png", image_name)),
            buffer,
            dimensions.width as u32,
            dimensions.height as u32,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }
    #[test]
    fn uv_field_0() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=3 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = DVec3::ZERO;
            let dir = DVec3::Z;
            let vertical_fov = 50.0;
            let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);

            let background = image::open("uv.png").unwrap();
            let stars = Stars::new(background);

            let black_hole_pos = 5.0 * DVec3::Z;
            let radius = 0.0;

            let black_hole = BlackHole::new(black_hole_pos, radius);
            render(&mut camera, &stars, &black_hole);

            write_image(
                &format!("uv_field_{}_size_{}", radius, dim),
                camera.get_colors(),
                camera.get_dimensions(),
            );
        }
        Ok(())
    }
    #[test]
    fn uv_field_1() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=4 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = DVec3::ZERO;
            let dir = DVec3::Z;
            let vertical_fov = 50.0;
            let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);

            let background = image::open("uv.png").unwrap();
            let stars = Stars::new(background);

            let black_hole_pos = 5.0 * DVec3::Z;
            let radius = 1.0;

            let black_hole = BlackHole::new(black_hole_pos, radius);
            render(&mut camera, &stars, &black_hole);

            write_image(
                &format!("uv_field_{}_size_{}", radius, dim),
                camera.get_colors(),
                camera.get_dimensions(),
            );
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_0() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=3 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = DVec3::ZERO;
            let dir = DVec3::Z;
            let vertical_fov = 50.0;
            let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);

            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let black_hole_pos = 5.0 * DVec3::Z;
            let radius = 0.0;

            let black_hole = BlackHole::new(black_hole_pos, radius);
            render(&mut camera, &stars, &black_hole);

            write_image(
                &format!("black_hole_field_{}_size_{}", radius, dim),
                camera.get_colors(),
                camera.get_dimensions(),
            );
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_1() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=4 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = DVec3::ZERO;
            let dir = DVec3::Z;
            let vertical_fov = 50.0;
            let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);

            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let black_hole_pos = 5.0 * DVec3::Z;
            let radius = 1.0;

            let black_hole = BlackHole::new(black_hole_pos, radius);
            render(&mut camera, &stars, &black_hole);

            write_image(
                &format!("black_hole_field_{}_size_{}", radius, dim),
                camera.get_colors(),
                camera.get_dimensions(),
            );
        }
        Ok(())
    }
}
