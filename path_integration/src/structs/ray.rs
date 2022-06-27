use glam::DVec3;
use quaternion::Quaternion;
#[derive(Debug, Clone)]
pub struct Ray {
    pub pos: DVec3,
    pub dir: DVec3,
    pub up: DVec3,
}

// Need to:
// 1. Rotate pos to (0.0,0.0,-Z)
// 2. Rotate up to (0.0,Y,0.0)
// 3. Rotate dir to (-X,0.0,-Z)
fn canonical_rotation(ray: &Ray) -> Quaternion<f64> {
    let rotate_pos: Quaternion<f64> =
        quaternion::rotation_from_to(ray.pos.to_array(), (-DVec3::Z).to_array());
    let up = quaternion::rotate_vector(rotate_pos, ray.up.to_array());
    let rotated_up = quaternion::rotation_from_to(up, DVec3::Y.to_array());
    let composite = quaternion::mul(rotated_up, rotate_pos);
    let dir = quaternion::rotate_vector(composite, ray.dir.to_array());

    // There are two different directions this could take. We need to calculate the actual angle of rotation.
    let angle = f64::atan2(dir[1], -dir[0]);
    let final_rot = quaternion::euler_angles(0.0, 0.0, angle);

    quaternion::mul(final_rot, composite)
}

fn to_canonical_direction(ray: &Ray) -> DVec3 {
    let q = canonical_rotation(ray);
    DVec3::from_array(quaternion::rotate_vector(q, ray.dir.to_array())).normalize()
}

// let's rotate the ray start to (0.0,0.0,-Z), then calculate
fn from_canonical_direction(dir: &DVec3, original_ray: &Ray) -> DVec3 {
    let q = canonical_rotation(original_ray);
    let q_len = quaternion::square_len(q);
    let q_inv = quaternion::scale(quaternion::conj(q), 1.0 / q_len);
    let rotated_dir = quaternion::rotate_vector(q_inv, dir.to_array());
    DVec3::from_array(rotated_dir).normalize()
}

impl Ray {
    pub fn new_with_up(pos: DVec3, dir: DVec3, up: DVec3) -> Self {
        Self {
            pos,
            dir: dir.normalize(),
            up,
        }
    }

    pub fn new(pos: DVec3, dir: DVec3) -> Self {
        Self {
            pos,
            dir: dir.normalize(),
            up: DVec3::Y,
        }
    }

    pub fn canonical_dir(&self) -> DVec3 {
        to_canonical_direction(self)
    }

    pub fn from_canonical_dir(&self, canonical_dir: &DVec3) -> DVec3 {
        from_canonical_direction(canonical_dir, self)
    }
}

#[cfg(test)]

mod tests {
    use glam::DVec3;

    use crate::Ray;

    #[test]
    fn ray_dir_returns_vec3() {
        let v = DVec3::new(1.0, -2.0, 3.0);
        let unit_v = DVec3::new(1.0, 1.0, 1.0);
        let ray = Ray::new(v, unit_v);

        assert_eq!(DVec3::new(1.0, 1.0, 1.0).normalize(), ray.dir);
    }

    #[test]
    fn to_canonical_direction_45_degrees() {
        let eps = 0.000001;
        let test_cases = [
            (
                Ray::new_with_up(-DVec3::Z, DVec3::Z + DVec3::X, DVec3::Y),
                (DVec3::Z - DVec3::X).normalize(),
            ),
            (
                Ray::new_with_up(-DVec3::Z, DVec3::Z - DVec3::X, DVec3::Y),
                (DVec3::Z - DVec3::X).normalize(),
            ),
            (
                Ray::new_with_up(-DVec3::Z, DVec3::Z + DVec3::Y, DVec3::Y),
                (DVec3::Z - DVec3::X).normalize(),
            ),
            (
                Ray::new_with_up(-DVec3::Z, DVec3::Z - DVec3::Y, DVec3::Y),
                (DVec3::Z - DVec3::X).normalize(),
            ),
            (
                Ray::new_with_up(
                    -DVec3::Z,
                    DVec3::Z + (DVec3::Y + DVec3::X).normalize(),
                    DVec3::Y,
                ),
                (DVec3::Z - DVec3::X).normalize(),
            ),
            (
                Ray::new_with_up(
                    -DVec3::Z,
                    DVec3::Z + (-DVec3::Y + DVec3::X).normalize(),
                    DVec3::Y,
                ),
                (DVec3::Z - DVec3::X).normalize(),
            ),
            (
                Ray::new_with_up(
                    -DVec3::Z,
                    DVec3::Z + (DVec3::Y - DVec3::X).normalize(),
                    DVec3::Y,
                ),
                (DVec3::Z - DVec3::X).normalize(),
            ),
            (
                Ray::new_with_up(
                    -DVec3::Z,
                    DVec3::Z + (-DVec3::Y - DVec3::X).normalize(),
                    DVec3::Y,
                ),
                (DVec3::Z - DVec3::X).normalize(),
            ),
        ];
        for (ray, canonical_dir) in test_cases {
            let dir = ray.canonical_dir();
            assert_eq!(
                (dir - canonical_dir).length() < eps,
                true,
                "\nExpected: {:?}\nActual: {:?}\nRay: {:?}",
                canonical_dir,
                dir,
                ray
            );
        }
    }

    #[test]
    fn to_canonical_direction_idempotent() {
        let eps = 0.000001;
        let test_cases = [
            Ray::new_with_up(-DVec3::Z, DVec3::Z + DVec3::X, DVec3::Y),
            Ray::new_with_up(-DVec3::Z, DVec3::Z - DVec3::X, DVec3::Y),
            Ray::new_with_up(-DVec3::Z, DVec3::Z + DVec3::Y, DVec3::Y),
            Ray::new_with_up(-DVec3::Z, DVec3::Z - DVec3::Y, DVec3::Y),
            Ray::new_with_up(
                -DVec3::Z,
                DVec3::Z + (DVec3::Y + DVec3::X).normalize(),
                DVec3::Y,
            ),
            Ray::new_with_up(
                -DVec3::Z,
                DVec3::Z + (-DVec3::Y + DVec3::X).normalize(),
                DVec3::Y,
            ),
            Ray::new_with_up(
                -DVec3::Z,
                DVec3::Z + (DVec3::Y - DVec3::X).normalize(),
                DVec3::Y,
            ),
            Ray::new_with_up(
                -DVec3::Z,
                DVec3::Z + (-DVec3::Y - DVec3::X).normalize(),
                DVec3::Y,
            ),
            Ray::new_with_up(-DVec3::Z, -DVec3::Z + DVec3::X, DVec3::Y),
            Ray::new_with_up(-DVec3::Z, -DVec3::Z - DVec3::X, DVec3::Y),
            Ray::new_with_up(-DVec3::Z, -DVec3::Z + DVec3::Y, DVec3::Y),
            Ray::new_with_up(-DVec3::Z, -DVec3::Z - DVec3::Y, DVec3::Y),
        ];
        for ray in test_cases {
            let dir = ray.canonical_dir();
            let original_dir = ray.from_canonical_dir(&dir);
            assert_eq!(
                (ray.dir - original_dir).length() < eps,
                true,
                "\nExpected: {:?}\nActual: {:?}\nRay: {:?}",
                ray.dir,
                original_dir,
                ray
            );
        }
    }

    #[test]
    fn forward_vector_invariant_to_up_rotation() {
        let eps = 0.000001;
        let test_cases = [
            Ray::new_with_up(-DVec3::Z, DVec3::Z, DVec3::Y),
            Ray::new_with_up(-DVec3::Z, DVec3::Z, DVec3::X),
            Ray::new_with_up(-DVec3::Z, DVec3::Z, -DVec3::Y),
            Ray::new_with_up(-DVec3::Z, DVec3::Z, -DVec3::X),
        ];
        for ray in test_cases {
            let dir = ray.canonical_dir();
            assert_eq!(
                (ray.dir - dir).length() < eps,
                true,
                "\nExpected: {:?}\nActual: {:?}\nRay: {:?}",
                ray.dir,
                dir,
                ray
            );
        }
    }
}
