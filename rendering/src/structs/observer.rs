use std::f64::consts::PI;

use glam::f64::DQuat;
use glam::DVec3;
use path_integration::BlackHole;

use super::data::Data;
pub struct Observer {
    canon_forward: DVec3,
    canon_up: DVec3,
    canon_right: DVec3,
    view_width: f64,
    from_canon: DQuat,
}

// We want to line up pos with -Z, up with +Y.
// We apply this rotation to forward and right to get the rays. Then use them to generate the initial dir.

fn canon_rotation(pos: DVec3, up: DVec3) -> (DQuat, DQuat) {
    let pos = pos.normalize();
    let up = up.normalize();

    let target_pos = -DVec3::Z;
    let target_up = DVec3::Y;

    let q1 = DQuat::from_rotation_arc(pos, target_pos);
    let rotated_up = q1 * up;
    let q2 = DQuat::from_rotation_arc(rotated_up, target_up);
    let to_canon = q2 * q1;
    let from_canon = to_canon.inverse();

    (to_canon, from_canon)
}

impl Observer {
    pub fn new(pos: DVec3, forward: DVec3, up: DVec3, vertical_fov_degrees: f64) -> Self {
        let (to_canon, from_canon) = canon_rotation(pos, up);
        let forward = forward.normalize();
        let canon_forward = (to_canon * forward).normalize();
        let view_mag = 2.0 * f64::tan(PI * vertical_fov_degrees / 360.0);
        let canon_up = DVec3::Y;
        let canon_right = canon_forward.cross(canon_up).normalize();
        Self {
            canon_forward,
            canon_up,
            canon_right,
            view_width: view_mag,
            from_canon,
        }
    }

    pub fn to_start_dir(&self, data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            match sample {
                Data::Sample(x, y, view_x, view_y) => {
                    *sample = Data::CanonDir(*x, *y, self.canon(*view_x, *view_y));
                }
                _ => {}
            }
        }
    }

    pub fn all_to_final_dir(&self, black_hole: &BlackHole, data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            match sample {
                Data::CanonDir(x, y, start_dir) => {
                    let fetch = black_hole.fetch_final_dir(start_dir.z);
                    if fetch.is_some() {
                        let test = self
                            .to_final_dir_transform(&start_dir, &fetch.unwrap())
                            .normalize();
                        *sample = Data::FinalDir(*x, *y, test);
                    } else {
                        *sample = Data::NoFinalDir(*x, *y);
                    }
                }
                _ => {
                    panic!("Should be canon dir format here!")
                }
            }
        }
    }

    pub fn to_final_dir(&self, view_x: f64, view_y: f64, black_hole: &BlackHole) -> Option<DVec3> {
        let canon = self.canon(view_x, view_y);
        let fetch = black_hole.fetch_final_dir(canon.z);
        if fetch.is_some() {
            let test = self
                .to_final_dir_transform(&canon, &fetch.unwrap())
                .normalize();
            return Some(test);
        }
        return fetch;
    }
    // note that this isn't rotated into the XZ plane.
    fn canon(&self, view_x: f64, view_y: f64) -> DVec3 {
        return (self.view_width
            * ((view_x - 0.5) * self.canon_right + (view_y - 0.5) * self.canon_up)
            + self.canon_forward)
            .normalize();
    }

    fn to_final_dir_transform(&self, canon: &DVec3, dir: &DVec3) -> DVec3 {
        let angle = f64::atan2(canon.y, -canon.x);
        let first_rot = DQuat::from_rotation_z(-angle);
        self.from_canon * first_rot * *dir
    }
}
