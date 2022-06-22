use super::dimensions::Dimensions;

pub struct Config {
    pub expected_image_size: Dimensions,
}

impl Config {
    pub fn new(expected_image_size: Dimensions) -> Self {
        Self {
            expected_image_size,
        }
    }
}
