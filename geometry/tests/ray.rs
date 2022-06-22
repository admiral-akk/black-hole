#[cfg(test)]

mod tests {
    use geometry::{Ray, UnitVec3, Vec3};

    #[test]
    fn ray_dir_returns_vec3() {
        let v = Vec3::new(1.0, -2.0, 3.0);
        let unit_v = UnitVec3::new(Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(v, unit_v);

        assert_eq!(UnitVec3::new(Vec3::new(1.0, 1.0, 1.0)).vec3(), ray.dir());
    }
}
