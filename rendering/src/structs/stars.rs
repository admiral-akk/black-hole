use std::f32::consts::{FRAC_1_PI, TAU};

use glam::Vec3;
use image::{DynamicImage, GenericImageView};

use crate::utils::extensions::ToPolar;

use super::{data::Data, polar_coordinates::PolarCoordinates};

pub struct Stars {
    background: DynamicImage,
    offset: PolarCoordinates,
}

const FRAC_1_TAU: f32 = 1.0 / TAU;

impl Stars {
    pub fn new(background: DynamicImage) -> Stars {
        Self {
            background,
            offset: PolarCoordinates {
                phi: 0.0,
                theta: 0.0,
            },
        }
    }

    pub fn update_position(&mut self, pos: &Vec3) {
        self.offset = pos.to_polar();
    }

    pub fn to_rgba(&self, data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            match sample {
                Data::Polar(index, polar) => {
                    let x_image = FRAC_1_TAU
                        * (self.background.width() as f32)
                        * (self.offset.phi as f32 + polar.phi as f32);
                    let y_image = FRAC_1_PI
                        * (self.background.height() as f32)
                        * ((self.offset.theta + polar.theta) + std::f32::consts::FRAC_PI_2);
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
