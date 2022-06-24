use crate::structs::dimensions::Dimensions;
use geometry::{Ray, Vec3};
use image::{GenericImageView, Pixel};

pub fn render(
    x: usize,
    y: usize,
    dimensions: &Dimensions,
    image: &image::DynamicImage,
    vertical_fov_degrees: f32,
    out: &mut [u8],
) {
    let y_size = f32::tan(vertical_fov_degrees * std::f32::consts::PI / 360.0);
    let x_size = y_size * dimensions.aspect_ratio();
    let (half_width, half_height) = (
        (dimensions.width / 2) as f32,
        (dimensions.height / 2) as f32,
    );
    let view_x = x_size * ((x as f32) - half_width) / half_width;
    let view_y = y_size * ((y as f32) - half_height) / half_height;

    // get angle from z-forward with y as pivot
    let viewport = Vec3::new(view_x, view_y, 1.0);
    let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), viewport);

    let (x, y, z) = ray.dir.xyz();
    let x_angle = f32::atan2(x, z);
    let y_angle = f32::atan2(-y, (x * x + z * z).sqrt());

    let x_image = (x_angle + std::f32::consts::PI) / std::f32::consts::TAU;
    let y_image = (y_angle + std::f32::consts::FRAC_PI_2) / std::f32::consts::PI;

    let i_dim = image.dimensions();
    let rgba = image
        .get_pixel(
            (((i_dim.0 - 1) as f32) * x_image) as u32,
            (((i_dim.1 - 1) as f32) * y_image) as u32,
        )
        .to_rgba();
    out[0] = rgba.0[0];
    out[1] = rgba.0[1];
    out[2] = rgba.0[2];
    out[3] = rgba.0[3];
}
