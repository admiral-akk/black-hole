use crate::structs::{config::Config, dimensions::Dimensions};

pub struct Renderer {
    config: Config,
}

impl Renderer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    fn render_uv(&self, buf: &mut [u8], dimensions: &Dimensions) {
        for y in (0..dimensions.height).rev() {
            for x in 0..dimensions.width {
                let r = (255 * x / dimensions.width) as u8;
                let g = (255 * y / dimensions.height) as u8;
                let index = 4 * dimensions.to_index(x, y);
                buf[index] = r;
                buf[index + 1] = g;
                buf[index + 2] = 0;
                buf[index + 3] = 255;
            }
        }
    }

    pub fn render(&self, buf: &mut [u8], dimensions: &Dimensions) {
        self.render_uv(buf, dimensions);
    }
}
