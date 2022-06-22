use crate::structs::dimensions::Dimensions;
use image::{GenericImageView, Pixel};

pub fn render(
    x: usize,
    y: usize,
    dimensions: &Dimensions,
    image: &image::DynamicImage,
    out: &mut [u8],
) {
    let i_dim = image.dimensions();
    let rgba = image
        .get_pixel(
            i_dim.0 * (x as u32) / (dimensions.width as u32),
            i_dim.1 * (y as u32) / (dimensions.height as u32),
        )
        .to_rgba();
    out[0] = rgba.0[0];
    out[1] = rgba.0[1];
    out[2] = rgba.0[2];
    out[3] = rgba.0[3];
}
