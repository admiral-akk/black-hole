use super::dimensions::Dimensions;

pub struct Config {
    pub image_size: Dimensions,
}

impl Config {
    pub fn new(image_size: Dimensions) -> Self {
        Self { image_size }
    }
}
