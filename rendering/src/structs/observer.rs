use std::f64::consts::PI;

use glam::DVec3;
use path_integration::Ray;

pub struct Observer {
    pos: DVec3,
    forward: DVec3,
    up: DVec3,
    right: DVec3,
}

impl Observer {
    pub fn new(pos: DVec3, forward: DVec3, up: DVec3, vertical_fov_degrees: f64) -> Self {
        // always face towards the black hole
        let forward = forward.normalize();
        let view_mag = 2.0 * f64::tan(PI * vertical_fov_degrees / 360.0);
        let up = view_mag * (up - forward.dot(up) * up.normalize()).normalize();
        let right = view_mag * forward.cross(up).normalize();
        Self {
            pos,
            forward,
            up,
            right,
        }
    }

    fn to_ray(&self, view_x: f64, view_y: f64) -> Ray {
        let dir =
            ((view_x - 0.5) * self.right + (view_y - 0.5) * self.up + self.forward).normalize();
        Ray::new_with_up(self.pos, dir, self.up.normalize())
    }

    pub fn to_rays(&self, view_positions: &Vec<(f64, f64)>) -> Vec<Ray> {
        let mut rays = Vec::new();
        for (view_x, view_y) in view_positions {
            rays.push(self.to_ray(*view_x, *view_y));
        }
        rays
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use glam::DVec3;
    use path_integration::BlackHole;

    use crate::{
        render::render,
        structs::{image_data::ImageData, stars::Stars},
    };

    use super::Observer;

    fn init(
        pos: DVec3,
        facing: DVec3,
        up: DVec3,
        vertical_fov: f64,
    ) -> (ImageData, Observer, Stars, BlackHole) {
        let dim = 800;
        let observer = Observer::new(pos, facing, up, vertical_fov);
        let image_data = ImageData::new(dim, dim);

        let background = image::open("uv.png").unwrap();
        let stars = Stars::new(background);

        let radius = 1.0;

        let black_hole = BlackHole::new(
            radius,
            pos.length(),
            vertical_fov * std::f64::consts::PI / 180.0,
        );
        (image_data, observer, stars, black_hole)
    }

    #[test]
    fn base_observer() -> Result<(), Box<dyn std::error::Error>> {
        let pos = -5.0 * DVec3::Z;
        let vertical_fov = 90.0;
        let (mut image_data, observer, stars, black_hole) = init(pos, -pos, DVec3::Y, vertical_fov);
        render(&mut image_data, &observer, &stars, &black_hole);

        let file_name = format!("observer/base_observer");
        image_data.write_image(&file_name);
        Ok(())
    }

    #[test]
    fn rotation_xz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * (f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::X);
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, black_hole) =
                init(pos, -pos, DVec3::Y, vertical_fov);
            render(&mut image_data, &observer, &stars, &black_hole);

            let file_name = format!("observer/observer_XZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_xy_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * (f64::cos(angle_rad) * DVec3::Y + f64::sin(angle_rad) * DVec3::X);
            let up = DVec3::Z;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, black_hole) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &black_hole);

            let file_name = format!("observer/observer_XY_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_yz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * (f64::cos(angle_rad) * DVec3::Y + f64::sin(angle_rad) * DVec3::Z);
            let up = DVec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, black_hole) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &black_hole);

            let file_name = format!("observer/observer_YZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_x_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::X;
            let up = f64::cos(angle_rad) * DVec3::Y + f64::sin(angle_rad) * DVec3::Z;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, black_hole) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &black_hole);

            let file_name = format!("observer/observer_rotate_up_X_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_y_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::Y;
            let up = f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, black_hole) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &black_hole);

            let file_name = format!("observer/observer_rotate_up_Y_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_z_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::Z;
            let up = f64::cos(angle_rad) * DVec3::Y + f64::sin(angle_rad) * DVec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, black_hole) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &black_hole);

            let file_name = format!("observer/observer_rotate_up_Z_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_view_XZ_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::Z;
            let dir = f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::X;
            let up = DVec3::Y;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, black_hole) = init(pos, dir, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &black_hole);

            let file_name = format!("observer/observer_rotate_view_XZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_view_YZ_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::Z;
            let dir = f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::Y;
            let up = DVec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, black_hole) = init(pos, dir, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &black_hole);

            let file_name = format!("observer/observer_rotate_view_YZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }
}
