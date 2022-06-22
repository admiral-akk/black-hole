#[cfg(test)]

mod tests {
    use geometry::{UnitVec3, Vec3};

    #[test]
    fn unit_vec3_is_normalized() {
        let v = &Vec3::new(1.0, -2.0, 3.0);

        let unit_v = UnitVec3::new((*v).clone());

        assert_eq!(v.normalized(), *unit_v.vec3());
    }
}
