use std::f32::consts::PI;

use glam::{Quat, Vec3, Vec3A};

use super::{data::Data, ray_cache::RAY_START_DIR};
pub struct Observer {
    forward: Vec3A,
    up: Vec3A,
    right: Vec3A,
    view_width: f32,
}

impl Observer {
    pub fn new(pos: Vec3, forward: Vec3, up: Vec3, vertical_fov_degrees: f32) -> Self {
        let q = Quat::from_rotation_arc(pos.normalize(), RAY_START_DIR);

        let forward = q * forward.normalize();
        let up = q * up.normalize();
        let right = forward.cross(up);

        let view_width = 2.0 * f32::tan(PI * vertical_fov_degrees / 360.0);
        Self {
            forward: Vec3A::from(forward),
            up: Vec3A::from(up),
            right: Vec3A::from(right),
            view_width,
        }
    }

    pub fn to_start_dir(&self, data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            match sample {
                Data::Sample(index, view_x, view_y) => {
                    *sample = Data::ObserverDir(*index, self.start_dir(*view_x, *view_y));
                }
                _ => {}
            }
        }
    }

    fn start_dir(&self, view_x: f32, view_y: f32) -> Vec3A {
        return self.view_width * ((view_x - 0.5) * self.right + (view_y - 0.5) * self.up)
            + self.forward;
    }
}
