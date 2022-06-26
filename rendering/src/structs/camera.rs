use glam::DVec3;
use path_integration::Ray;

use super::{dimensions::Dimensions, image_data::ImageData, observer::Observer};

pub struct Camera {
    dimensions: Dimensions,
    observer: Observer,
    image_data: ImageData,
}

impl Camera {
    pub fn new(dimensions: Dimensions, pos: DVec3, vertical_fov: f64) -> Self {
        let observer = Observer::new(pos, DVec3::Z, DVec3::Y, vertical_fov);
        let image_data = ImageData::new(dimensions.width, dimensions.height);
        Self {
            dimensions,
            observer,
            image_data,
        }
    }
    pub fn get_rays(&self, x: usize, y: usize) -> Vec<Ray> {
        let view_positions = self.image_data.get_samples(x, y);
        let mut rays = Vec::new();
        for (view_x, view_y) in view_positions {
            rays.push(self.observer.to_ray(view_x, view_y));
        }
        rays
    }

    pub fn write_color(&mut self, x: usize, y: usize, color: &[u8; 4]) {
        self.image_data.add_sample(x, y, color);
    }

    pub fn get_colors(&mut self) -> &[u8] {
        &self.image_data.get_image()
    }

    pub fn get_dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}
