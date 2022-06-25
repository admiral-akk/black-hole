use crate::DVec3;

pub fn to_phi_theta(vec: &DVec3) -> (f64, f64) {
    let horizontal_len = (vec.x * vec.x + vec.y * vec.y).sqrt();
    (
        (f64::atan2(vec.y, vec.x) + std::f64::consts::TAU) % std::f64::consts::TAU,
        f64::atan2(vec.z, horizontal_len),
    )
}
