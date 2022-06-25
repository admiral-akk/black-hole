mod structs;

pub type Vector3 = glam::Vec3;
pub type DVector3 = glam::DVec3;
pub type Vec3 = Vector3;
pub type DVec3 = DVector3;

pub fn to_phi_theta(vec: &DVec3) -> (f64, f64) {
    let horizontal_len = (vec.x * vec.x + vec.y * vec.y).sqrt();
    (f64::atan2(vec.x, vec.y), f64::atan2(vec.z, horizontal_len))
}

pub use structs::disc::Disc;
pub use structs::ray::Ray;
pub use structs::sphere::Sphere;
