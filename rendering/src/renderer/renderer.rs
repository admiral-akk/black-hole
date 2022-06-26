use ::image::DynamicImage;

use crate::structs::dimensions::Dimensions;

use super::{image, ray_skybox, skybox, uv};

pub struct Renderer {}

pub enum RenderType {
    UV,
    Skybox {
        vertical_fov_degrees: f32,
    },
    Image {
        image: DynamicImage,
    },
    RaySkybox {
        vertical_fov_degrees: f32,
        image: DynamicImage,
    },
}

pub struct RenderConfig {
    pub dimensions: Dimensions,
    pub render_type: RenderType,
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, buf: &mut [u8], config: &RenderConfig) {
        let dimensions = &config.dimensions;
        for y in (0..dimensions.height).rev() {
            for x in 0..dimensions.width {
                let index = 4 * dimensions.to_index(x, y);
                let out = &mut buf[index..(index + 4)];
                match &config.render_type {
                    RenderType::UV => uv::uv_renderer::render(x, y, dimensions, out),
                    RenderType::Skybox {
                        vertical_fov_degrees,
                    } => skybox::skybox_renderer::render(
                        x,
                        y,
                        dimensions,
                        *vertical_fov_degrees,
                        out,
                    ),
                    RenderType::Image { image } => {
                        image::image_renderer::render(x, y, dimensions, image, out)
                    }
                    RenderType::RaySkybox {
                        vertical_fov_degrees,
                        image,
                    } => ray_skybox::ray_skybox_renderer::render(
                        x,
                        y,
                        dimensions,
                        image,
                        *vertical_fov_degrees,
                        out,
                    ),
                }
            }
        }
    }
}
