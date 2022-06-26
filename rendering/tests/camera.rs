#[cfg(test)]
mod tests {
    use std::path::Path;

    use glam::DVec3;
    use image::GenericImageView;
    use rendering::structs::{camera::Camera, dimensions::Dimensions};

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
    fn rays() {
        let (width, height) = (10, 10);
        let dimensions = Dimensions::new(width, height);
        let pos = DVec3::new(0.0, 0.0, 0.0);
        let vertical_fov = 20.0;
        let camera = Camera::new(dimensions, pos, vertical_fov);

        let rays = camera.get_rays();
        for index in 0..rays.len() {
            let (x, y) = camera.get_dimensions().to_xy(index);
            println!("({},{}): {:?}", x, y, rays[index]);
        }
    }

    #[test]
    fn write_uv() {
        let (width, height) = (1024, 1024);
        let dimensions = Dimensions::new(width, height);
        let pos = DVec3::new(0.0, 0.0, 0.0);
        let vertical_fov = 20.0;
        let mut camera = Camera::new(dimensions, pos, vertical_fov);

        let uv = image::open("uv.png").unwrap();

        for index in 0..camera.get_dimensions().size() {
            let (x, y) = camera.get_dimensions().to_xy(index);
            let c = uv.get_pixel(x as u32, y as u32);
            camera.write_color(index, &c.0);
        }

        write_image("uv_copy", camera.get_colors(), camera.get_dimensions());
    }
}
