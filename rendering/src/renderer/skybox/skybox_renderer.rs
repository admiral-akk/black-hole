use geometry::{Ray, UnitVec3, Vec3};

use crate::structs::dimensions::Dimensions;

pub fn render(
    x: usize,
    y: usize,
    dimensions: &Dimensions,
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

    let viewport = Vec3::new(view_x, view_y, 1.0);
    let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), UnitVec3::new(viewport));

    let r = (255.0 * ray.dir.xyz().0) as u8;
    let g = (255.0 * ray.dir.xyz().1) as u8;
    out[0] = r;
    out[1] = g;
    out[2] = 0;
    out[3] = 255;
}
