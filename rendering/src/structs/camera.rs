use glam::DVec3;

use super::{dimensions::Dimensions, image_data::ImageData, observer::Observer};

pub struct Camera {
    observer: Observer,
    image_data: ImageData,
}

impl Camera {
    pub fn new(dimensions: Dimensions, pos: DVec3, vertical_fov: f64) -> (Observer, ImageData) {
        let observer = Observer::new(pos, DVec3::Z, DVec3::Y, vertical_fov);
        let image_data = ImageData::new(dimensions.width, dimensions.height);
        (observer, image_data)
    }
}
