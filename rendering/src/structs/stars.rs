use geometry::{utils::to_phi_theta, DVec3};
use image::{DynamicImage, GenericImageView};

pub struct Stars {
    background: DynamicImage,
}

impl Stars {
    pub fn new(background: DynamicImage) -> Self {
        Self { background }
    }

    pub fn get_rgba(&self, dir: &DVec3) -> [u8; 4] {
        let mod_dir = DVec3::new(dir.x, dir.z, dir.y);
        let (phi, theta) = to_phi_theta(&mod_dir);
        let x = (self.background.width() as f64) * phi / std::f64::consts::TAU;
        let y = (self.background.height() as f64) * (theta + std::f64::consts::FRAC_PI_2)
            / std::f64::consts::PI;
        let rgba = self.background.get_pixel(
            (x as u32) % self.background.width(),
            (y as u32) % self.background.height(),
        );

        rgba.0
    }
}
