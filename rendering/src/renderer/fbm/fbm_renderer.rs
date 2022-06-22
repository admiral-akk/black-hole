use crate::structs::dimensions::Dimensions;

pub fn render(x: usize, y: usize, dimensions: &Dimensions, out: &mut [u8]) {
    let r = (255 * x / (dimensions.width - 1)) as u8;
    let g = (255 * y / (dimensions.height - 1)) as u8;
    out[0] = r;
    out[1] = g;
    out[2] = 0;
    out[3] = 255;
}
