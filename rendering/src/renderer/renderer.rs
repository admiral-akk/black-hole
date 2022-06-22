use crate::structs::dimensions::Dimensions;

use super::{fbm, skybox, uv};

pub struct Renderer {}

pub enum RenderType {
    UV,
    Skybox { vertical_fov_degrees: f32 },
    fBM,
}

pub struct RenderConfig {
    pub dimensions: Dimensions,
    pub render_type: RenderType,
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    fn fBM(&self, _buf: &mut [u8], dimensions: &Dimensions) {
        for _y in (0..dimensions.height).rev() {
            for _x in 0..dimensions.width {}
        }
    }

    pub fn render(&self, buf: &mut [u8], config: &RenderConfig) {
        let dimensions = &config.dimensions;
        for y in (0..dimensions.height).rev() {
            for x in 0..dimensions.width {
                let index = 4 * dimensions.to_index(x, y);
                let out = &mut buf[index..(index + 4)];
                match config.render_type {
                    RenderType::UV => uv::uv_renderer::render(x, y, dimensions, out),
                    RenderType::Skybox {
                        vertical_fov_degrees,
                    } => {
                        skybox::skybox_renderer::render(x, y, dimensions, vertical_fov_degrees, out)
                    }
                    RenderType::fBM => fbm::fbm_renderer::render(x, y, dimensions, out),
                }
            }
        }
    }
}
