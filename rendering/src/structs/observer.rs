use std::f32::consts::PI;

use glam::{Quat, Vec3};

use crate::utils::extensions::ToPolar;

use super::{data::Data, ray_cache::RayCache};
pub struct Observer {
    canon_forward: Vec3,
    canon_up: Vec3,
    canon_right: Vec3,
    view_width: f32,
    from_canon: Quat,
}

// We want to line up pos with -Z, up with +Y.
// We apply this rotation to forward and right to get the rays. Then use them to generate the initial dir.

fn canon_rotation(pos: Vec3, up: Vec3) -> (Quat, Quat) {
    let pos = pos.normalize();
    let up = up.normalize();

    let target_pos = -Vec3::Z;
    let target_up = Vec3::Y;

    let q1 = Quat::from_rotation_arc(pos, target_pos);
    let rotated_up = q1 * up;
    let q2 = Quat::from_rotation_arc(rotated_up, target_up);
    let to_canon = q2 * q1;
    let from_canon = to_canon.inverse();

    (to_canon, from_canon)
}

impl Observer {
    pub fn new(pos: Vec3, forward: Vec3, up: Vec3, vertical_fov_degrees: f32) -> Self {
        let (to_canon, from_canon) = canon_rotation(pos, up);
        let forward = forward.normalize();
        let canon_forward = (to_canon * forward).normalize();
        let view_width = 2.0 * f32::tan(PI * vertical_fov_degrees / 360.0);
        let canon_up = Vec3::Y;
        let canon_right = canon_forward.cross(canon_up).normalize();
        Self {
            canon_forward,
            canon_up,
            canon_right,
            view_width,
            from_canon,
        }
    }

    pub fn to_start_dir(&self, data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            match sample {
                Data::Sample(index, view_x, view_y) => {
                    *sample = Data::CanonDir(*index, self.canon(*view_x, *view_y));
                }
                _ => {}
            }
        }
    }

    pub fn all_to_final_dir(&self, ray_cache: &RayCache, data: &mut Vec<Data>) {
        let mut empty_index = 0_usize;

        for i in 0..data.len() {
            match data[i] {
                Data::CanonDir(index, start_dir) => {
                    let fetch = ray_cache.fetch_final_dir(start_dir.z as f32);
                    if fetch.is_some() {
                        let test = self.to_final_dir_transform(&start_dir, &fetch.unwrap());
                        data[empty_index] = Data::Polar(index, test.to_polar());
                        empty_index += 1;
                    }
                }
                _ => {
                    panic!("Should be canon dir format here!")
                }
            }
        }

        data.drain(empty_index..data.len());
    }

    // note that this isn't rotated into the XZ plane.
    fn canon(&self, view_x: f32, view_y: f32) -> Vec3 {
        return (self.view_width
            * ((view_x - 0.5) * self.canon_right + (view_y - 0.5) * self.canon_up)
            + self.canon_forward)
            .normalize();
    }

    fn to_final_dir_transform(&self, canon: &Vec3, dir: &Vec3) -> Vec3 {
        let angle = fast_math::atan2(canon.y, -canon.x);
        let first_rot = Quat::from_rotation_z(-angle);
        self.from_canon * first_rot * *dir
    }
}
