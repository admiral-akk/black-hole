use std::f32::consts::PI;

use glam::{Quat, Vec3};

use super::{data::Data, ray_cache::RAY_START_DIR};
pub struct Observer {
    forward: Vec3,
    up: Vec3,
    right: Vec3,
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
            forward,
            up,
            right,
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

    fn start_dir(&self, view_x: f32, view_y: f32) -> Vec3 {
        return (self.view_width * ((view_x - 0.5) * self.right + (view_y - 0.5) * self.up)
            + self.forward)
            .normalize();
    }
}
