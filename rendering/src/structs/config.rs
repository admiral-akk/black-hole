use super::dimensions::Dimensions;

pub struct Config {
    image_size: Dimensions,
}

impl Config {
    pub fn new(image_size: Dimensions) -> Config {
        Config { image_size }
    }
}
