#[cfg(test)]
mod tests {
    use std::path::Path;

    use geometry::DVec3;
    use image::GenericImageView;
    use rendering::structs::{camera::Camera, dimensions::Dimensions, stars::Stars};

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
    fn get_rgba() {
        let uv = image::open("uv.png").unwrap();
        let (width, height) = (uv.width() as usize, uv.height() as usize);
        let stars = Stars::new(uv);

        let mut image = Vec::new();

        for x in 0..width {
            for y in 0..height {
                let (phi, theta) = (
                    std::f64::consts::TAU * (x as f64) / (width as f64),
                    std::f64::consts::PI * (y as f64) / (height as f64)
                        - std::f64::consts::FRAC_PI_2,
                );

                let cos_theta = f64::cos(theta);

                let dir = DVec3::new(
                    cos_theta * f64::cos(phi),
                    cos_theta * f64::sin(phi),
                    f64::sin(theta),
                )
                .normalize();
                let c = stars.get_rgba(&dir);
                c.into_iter().for_each(|c| image.push(c));
            }
        }

        write_image("uv_stars", &image, &Dimensions::new(width, height))
    }
}
