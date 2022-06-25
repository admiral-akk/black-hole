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
    fn uv() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=3 {
            let dim = 10_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = DVec3::ZERO;
            let dir = DVec3::Z;
            let vertical_fov = 120.0;
            let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);

            let background = image::open("uv.png").unwrap();
            let stars = Stars::new(background);

            let black_hole_pos = 5.0 * DVec3::Z;
            let black_hole_magnitude = 0.0;

            let black_hole = BlackHole::new(black_hole_pos, black_hole_magnitude);
            render(&mut camera, &stars, &black_hole);

            write_image(
                &format!("uv_size_{}", dim),
                camera.get_colors(),
                camera.get_dimensions(),
            );
        }
        Ok(())
    }

    #[test]
    fn no_black_hole() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=3 {
            let dim = 10_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = DVec3::ZERO;
            let dir = DVec3::Z;
            let vertical_fov = 120.0;
            let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);

            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let black_hole_pos = 5.0 * DVec3::Z;
            let black_hole_magnitude = 0.0;

            let black_hole = BlackHole::new(black_hole_pos, black_hole_magnitude);
            render(&mut camera, &stars, &black_hole);

            write_image(
                &format!("black_hole_field_{}_size_{}", black_hole_magnitude, dim),
                camera.get_colors(),
                camera.get_dimensions(),
            );
        }
        Ok(())
    }

    #[test]
    fn no_small_hole() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=3 {
            let dim = 10_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = DVec3::ZERO;
            let dir = DVec3::Z;
            let vertical_fov = 120.0;
            let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);

            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let black_hole_pos = 5.0 * DVec3::Z;
            let black_hole_magnitude = 1.0;

            let black_hole = BlackHole::new(black_hole_pos, black_hole_magnitude);
            render(&mut camera, &stars, &black_hole);

            write_image(
                &format!("black_hole_field_{}_size_{}", black_hole_magnitude, dim),
                camera.get_colors(),
                camera.get_dimensions(),
            );
        }
        Ok(())
    }
}
