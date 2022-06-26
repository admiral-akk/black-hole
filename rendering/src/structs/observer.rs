use glam::DVec3;

pub struct Observer {
    pos: DVec3,
    forward: DVec3,
    up: DVec3,
    right: DVec3,
}

impl Observer {
    pub fn new(pos: DVec3, forward: DVec3, up: DVec3, vertical_fov_degrees: f64) -> Self {
        let dir = forward.normalize();
        let view_mag = f64::tan(std::f64::consts::PI * vertical_fov_degrees / 360.0);
        let up = view_mag * (up - up.dot(dir.normalize()) * up.normalize());
        let right = view_mag * dir.cross(up).normalize();
        Self {
            pos,
            forward,
            up,
            right,
        }
    }

    pub fn to_ray(&self, view_x: f64, view_y: f64) -> (DVec3, DVec3) {
        (
            self.pos,
            (view_x - 0.5) * self.right + (view_y - 0.5) * self.up + self.forward,
        )
    }
}
