use glam::DVec3;
use path_integration::Ray;

pub struct Observer {
    pos: DVec3,
    forward: DVec3,
    up: DVec3,
    right: DVec3,
}

impl Observer {
    pub fn new(pos: DVec3, up: DVec3, vertical_fov_degrees: f64) -> Self {
        // always face towards the black hole
        let forward = -1.0 * pos.normalize();
        let view_mag = 2.0 * f64::tan(std::f64::consts::PI * vertical_fov_degrees / 360.0);
        let up = view_mag * (up - up.dot(forward.normalize()) * up.normalize());
        let right = view_mag * forward.cross(up).normalize();
        Self {
            pos,
            forward,
            up,
            right,
        }
    }

    fn to_ray(&self, view_x: f64, view_y: f64) -> Ray {
        Ray::new(
            self.pos,
            (view_x - 0.5) * self.right + (view_y - 0.5) * self.up + self.forward,
        )
    }

    pub fn to_rays(&self, view_positions: &Vec<(f64, f64)>) -> Vec<Ray> {
        let mut rays = Vec::new();
        for (view_x, view_y) in view_positions {
            rays.push(self.to_ray(*view_x, *view_y));
        }
        rays
    }
}
