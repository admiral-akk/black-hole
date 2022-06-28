use std::f64::consts::PI;

use glam::DVec3;
use path_integration::BlackHole;
use quaternion::Quaternion;

pub struct Observer {
    canon_forward: DVec3,
    canon_up: DVec3,
    canon_right: DVec3,
    view_width: f64,
    from_canon: Quaternion<f64>,
}

// We want to line up pos with -Z, up with +Y.
// We apply this rotation to forward and right to get the rays. Then use them to generate the initial dir.

fn canon_rotation(pos: DVec3, up: DVec3) -> (Quaternion<f64>, Quaternion<f64>) {
    let target_pos = -pos.length() * DVec3::Z;
    let target_up = DVec3::Y;

    let q1 = quaternion::rotation_from_to(pos.to_array(), target_pos.to_array());
    let rotated_up = quaternion::rotate_vector(q1, up.to_array());
    let q2 = quaternion::rotation_from_to(rotated_up, target_up.to_array());
    let to_canon = quaternion::mul(q2, q1);
    let len = quaternion::square_len(to_canon);
    let from_canon = quaternion::scale(quaternion::conj(to_canon), 1.0 / len);
    (to_canon, from_canon)
}

impl Observer {
    pub fn new(pos: DVec3, forward: DVec3, up: DVec3, vertical_fov_degrees: f64) -> Self {
        let (to_canon, from_canon) = canon_rotation(pos, up);
        let forward = forward.normalize();
        let canon_forward =
            DVec3::from_array(quaternion::rotate_vector(to_canon, forward.to_array())).normalize();
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
        let final_rot = quaternion::euler_angles(0.0, 0.0, -angle);
        let inv = quaternion::rotate_vector(final_rot, dir.to_array());
        DVec3::from_array(quaternion::rotate_vector(self.from_canon, inv))
    }
}
