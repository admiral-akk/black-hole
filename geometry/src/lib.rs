mod structs;

pub type Vector3 = glam::Vec3;
pub type DVector3 = glam::DVec3;
pub type Vec3 = Vector3;
pub type DVec3 = DVector3;

pub use structs::disc::Disc;
pub use structs::ray::Ray;
pub use structs::sphere::Sphere;
