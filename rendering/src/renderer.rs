use crate::structs::dimensions::Dimensions;

pub struct Renderer {}

pub enum RenderType {
    UV,
}

pub struct RenderConfig {
    pub dimensions: Dimensions,
    pub render_type: RenderType,
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    fn render_uv(&self, buf: &mut [u8], config: &RenderConfig) {
        for y in (0..config.dimensions.height).rev() {
            for x in 0..config.dimensions.width {
                let r = (255 * x / (config.dimensions.width - 1)) as u8;
                let g = (255 * y / (config.dimensions.height - 1)) as u8;
                let index = 4 * config.dimensions.to_index(x, y);
                buf[index] = r;
                buf[index + 1] = g;
                buf[index + 2] = 0;
                buf[index + 3] = 255;
            }
        }
    }

    pub fn render(&self, buf: &mut [u8], config: &RenderConfig) {
        self.render_uv(buf, config);
    }
}
