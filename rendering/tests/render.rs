#[cfg(test)]

mod tests {
    use std::path::Path;

    use glam::DVec3;
    use path_integration::BlackHole;
    use rendering::{
        render::render,
        structs::{
            camera::Camera, dimensions::Dimensions, image_data::ImageData, observer::Observer,
            stars::Stars,
        },
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
            let dimensions2 = dimensions.clone();
            let pos = -5.0 * DVec3::Z;
            let vertical_fov = 50.0;
            let observer = Observer::new(pos, DVec3::Z, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);

            let background = image::open("uv.png").unwrap();
            let stars = Stars::new(background);

            let radius = 0.0;

            let black_hole =
                BlackHole::new(radius, &pos, vertical_fov * std::f64::consts::PI / 180.0);
            render(&mut image_data, &observer, &stars, &black_hole);

            write_image(
                &format!("uv_field_{}_size_{}", radius, dim),
                image_data.get_image(),
                &dimensions2,
            );
        }
        Ok(())
    }
    #[test]
    fn uv_field_1() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=4 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let dimensions2 = dimensions.clone();
            let pos = -5.0 * DVec3::Z;
            let vertical_fov = 50.0;
            let observer = Observer::new(pos, DVec3::Z, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);

            let background = image::open("uv.png").unwrap();
            let stars = Stars::new(background);

            let radius = 1.0;

            let black_hole =
                BlackHole::new(radius, &pos, vertical_fov * std::f64::consts::PI / 180.0);
            render(&mut image_data, &observer, &stars, &black_hole);

            write_image(
                &format!("uv_field_{}_size_{}", radius, dim),
                image_data.get_image(),
                &dimensions2,
            );
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_0() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=3 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let dimensions2 = dimensions.clone();
            let pos = -5.0 * DVec3::Z;
            let vertical_fov = 50.0;
            let observer = Observer::new(pos, DVec3::Z, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);

            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let radius = 0.0;

            let black_hole =
                BlackHole::new(radius, &pos, vertical_fov * std::f64::consts::PI / 180.0);
            render(&mut image_data, &observer, &stars, &black_hole);

            write_image(
                &format!("black_hole_field_{}_size_{}", radius, dim),
                image_data.get_image(),
                &dimensions2,
            );
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_1() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=4 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let dimensions2 = dimensions.clone();
            let pos = -5.0 * DVec3::Z;
            let vertical_fov = 50.0;

            let observer = Observer::new(pos, DVec3::Z, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);
            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let radius = 1.0;

            let black_hole =
                BlackHole::new(radius, &pos, vertical_fov * std::f64::consts::PI / 180.0);
            render(&mut image_data, &observer, &stars, &black_hole);

            write_image(
                &format!("black_hole_field_{}_size_{}", radius, dim),
                image_data.get_image(),
                &dimensions2,
            );
        }
        Ok(())
    }
}
