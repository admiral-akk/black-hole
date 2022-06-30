use std::f32::consts::{FRAC_1_PI, FRAC_PI_2, TAU};

use glam::Vec3;
use image::{DynamicImage, GenericImageView};

use crate::utils::extensions::ToPolar;

use super::{data::Data, polar_coordinates::PolarCoordinates};

pub struct Stars {
    background: DynamicImage,
    offset: PolarCoordinates,
    scaling: ScalingFactors,
}

struct ScalingFactors {
    pub background_width_f32: f32,
    pub background_height_f32: f32,
}

const FRAC_1_TAU: f32 = 1.0 / TAU;

impl Stars {
    pub fn new(background: DynamicImage) -> Stars {
        let (width, height) = background.dimensions();
        Self {
            background,
            offset: PolarCoordinates {
                phi: 0.0 + TAU,
                theta: FRAC_PI_2 + TAU,
            },
            scaling: ScalingFactors {
                background_width_f32: (width as f32) * FRAC_1_TAU,
                background_height_f32: (height as f32) * FRAC_1_PI,
            },
        }
    }

    pub fn update_position(&mut self, pos: &Vec3) {
        self.offset = pos.to_polar();
        self.offset.phi += TAU;
        self.offset.theta += FRAC_PI_2 + TAU;
    }

    pub fn to_rgba(&self, data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            let (phi, theta) = sample.get_polar();
            let x_image = self.scaling.background_width_f32 * (self.offset.phi + phi);
            let y_image = self.scaling.background_height_f32 * (self.offset.theta + theta);
            let rgba = self.background.get_pixel(
                (x_image as u32) % self.background.width(),
                (y_image as u32) % self.background.height(),
            );
            sample.set_color(&rgba.0);
        }
    }
}
