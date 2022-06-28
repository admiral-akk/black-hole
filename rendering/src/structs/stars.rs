use glam::Vec3;
use image::{DynamicImage, GenericImageView};

use super::{data::Data, polar_coordinates::PolarCoordinates};

pub struct Stars {
    background: DynamicImage,
}
impl Stars {
    pub fn new(background: DynamicImage) -> Self {
        Self { background }
    }

    pub fn to_rgba(&self, data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            match sample {
                Data::Polar(index, polar) => {
                    let x_image = (self.background.width() as f64) * (polar.phi as f64)
                        / std::f64::consts::TAU;
                    let y_image = (self.background.height() as f64) * (polar.theta as f64)
                        / std::f64::consts::PI;
                    let rgba = self.background.get_pixel(
                        (x_image as u32) % self.background.width(),
                        (y_image as u32) % self.background.height(),
                    );
                    *sample = Data::RGBA(*index, rgba.0);
                }
                _ => {}
            }
        }
    }

    pub fn get_rgba(&self, dir: &Vec3) -> [u8; 4] {
        let polar = PolarCoordinates::new(&dir);
        let x = (self.background.width() as f64) * (polar.phi as f64) / std::f64::consts::TAU;
        let y = (self.background.height() as f64) * (polar.theta as f64) / std::f64::consts::PI;
        let rgba = self.background.get_pixel(
            (x as u32) % self.background.width(),
            (y as u32) % self.background.height(),
        );

        rgba.0
    }
}
