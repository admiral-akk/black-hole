#[cfg(test)]

mod tests {

    use glam::DVec3;
    use path_integration::Field;
    use rendering::{
        render::render,
        structs::{
            dimensions::Dimensions, image_data::ImageData, observer::Observer, ray_cache::RayCache,
            stars::Stars,
        },
    };

    #[test]
    fn uv_field_0() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=3 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = -5.0 * DVec3::Z;
            let vertical_fov = 50.0;
            let observer = Observer::new(pos, -pos, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);

            let background = image::open("uv.png").unwrap();
            let stars = Stars::new(background);

            let radius = 0.0;

            let field = Field::new(radius, &pos);
            let ray_cache = RayCache::compute_new(10000, &field, pos.length());
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("uv_field_{}_size_{}", radius, dim);
            image_data.write_image(&file_name);
        }
        Ok(())
    }
    #[test]
    fn uv_field_1() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=4 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = -5.0 * DVec3::Z;
            let vertical_fov = 50.0;
            let observer = Observer::new(pos, -pos, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);

            let background = image::open("uv.png").unwrap();
            let stars = Stars::new(background);

            let radius = 1.0;

            let field = Field::new(radius, &pos);
            let ray_cache = RayCache::compute_new(10000, &field, pos.length());
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("uv_field_{}_size_{}", radius, dim);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_0() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=3 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = -5.0 * DVec3::Z;
            let vertical_fov = 50.0;
            let observer = Observer::new(pos, -pos, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);

            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let radius = 0.0;

            let field = Field::new(radius, &pos);
            let ray_cache = RayCache::compute_new(10000, &field, pos.length());
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("black_hole_field_{}_size_{}", radius, dim);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_1() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=4 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = -5.0 * DVec3::Z;
            let vertical_fov = 50.0;

            let observer = Observer::new(pos, -pos, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);
            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let radius = 1.0;

            let field = Field::new(radius, &pos);
            let ray_cache = RayCache::compute_new(10000, &field, pos.length());
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("black_hole_field_{}_size_{}", radius, dim);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_1_below() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=4 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = -5.0 * DVec3::Y;
            let vertical_fov = 50.0;

            let observer = Observer::new(pos, -pos, DVec3::Z, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);
            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let radius = 1.0;

            let field = Field::new(radius, &pos);
            let ray_cache = RayCache::compute_new(10000, &field, pos.length());
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("black_hole_field_{}_size_{}_below", radius, dim);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_1_left() -> Result<(), Box<dyn std::error::Error>> {
        for size_pow in 1..=4 {
            let dim = 50 * 2_usize.pow(size_pow);
            let dimensions = Dimensions::new(dim, dim);
            let pos = -5.0 * DVec3::X;
            let vertical_fov = 50.0;

            let observer = Observer::new(pos, -pos, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);
            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);

            let radius = 1.0;

            let field = Field::new(radius, &pos);
            let ray_cache = RayCache::compute_new(10000, &field, pos.length());
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("black_hole_field_{}_size_{}_left", radius, dim);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn black_hole_field_rotated_start() -> Result<(), Box<dyn std::error::Error>> {
        let vertical_fov = 75.0;
        let radius = 1.5;

        let field = Field::new(radius, &(5.0 * DVec3::Z));
        let ray_cache = RayCache::compute_new(10000, &field, 5.0);
        for i in 0..1 {
            let dim = 50 * 2_usize.pow(4);
            let dimensions = Dimensions::new(dim, dim);
            let angle = i as f64;
            let rad_angle = std::f64::consts::PI * angle / (3.0 * 180.0);
            let pos = 5.0 * (DVec3::X * f64::cos(rad_angle) + DVec3::Z * f64::sin(rad_angle));

            let observer = Observer::new(pos, -pos, DVec3::Y, vertical_fov);
            let mut image_data = ImageData::new(dimensions.width, dimensions.height);
            let background = image::open("starmap_2020_4k_gal.exr").unwrap();
            let stars = Stars::new(background);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("black_hole_field_{}_size_{}_angle_{}", radius, dim, angle);
            image_data.write_image(&file_name);
        }

        Ok(())
    }
}
