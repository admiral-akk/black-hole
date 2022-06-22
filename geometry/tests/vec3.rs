#[cfg(test)]

mod tests {
    use geometry::Vec3;

    #[test]
    fn vec3_mul() {
        let v = &Vec3::new(1.0, -2.0, 3.0);

        assert_eq!(2.0 * v, Vec3::new(2.0, -4.0, 6.0));
        assert_eq!(-2.0 * v, Vec3::new(-2.0, 4.0, -6.0));
        assert_eq!(0.0 * v, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(v * 2.0, Vec3::new(2.0, -4.0, 6.0));
        assert_eq!(v * -2.0, Vec3::new(-2.0, 4.0, -6.0));
        assert_eq!(v * 0.0, Vec3::new(0.0, 0.0, 0.0));
    }
    #[test]
    fn vec3_sub() {
        let v1 = &Vec3::new(1.0, -2.0, 3.0);
        let v2 = &Vec3::new(4.0, -1.0, 0.0);

        assert_eq!(v1 - v2, Vec3::new(-3.0, -1.0, 3.0));
    }

    #[test]
    fn vec3_dot() {
        let v1 = &Vec3::new(1.0, -2.0, 3.0);
        let v2 = &Vec3::new(4.0, -1.0, 0.0);

        assert_eq!(v1.dot(v2), 6.0);
    }

    #[test]
    fn vec3_div() {
        let v1 = &Vec3::new(1.0, -2.0, 3.0);

        assert_eq!(v1 / 1.0, Vec3::new(1.0, -2.0, 3.0));
        assert_eq!(v1 / 2.0, Vec3::new(0.5, -1.0, 1.5));
        assert_eq!(v1 / 10.0, Vec3::new(0.1, -0.2, 0.3));
        assert_eq!(v1 / -10.0, Vec3::new(-0.1, 0.2, -0.3));
    }

    #[test]
    fn vec3_normalized() {
        let v1 = &Vec3::new(1.0, -1.0, 0.0);

        assert_eq!(
            v1.normalized(),
            Vec3::new(1.0 / 2.0_f32.sqrt(), -1.0 / 2.0_f32.sqrt(), 0.0)
        );
    }

    #[test]
    fn vec3_normalize() {
        let v1 = &mut Vec3::new(1.0, -1.0, 0.0);

        v1.normalize();

        assert_eq!(
            *v1,
            Vec3::new(1.0 / 2.0_f32.sqrt(), -1.0 / 2.0_f32.sqrt(), 0.0)
        );
    }

    #[test]
    fn vec3_xyz() {
        let v1 = &Vec3::new(1.0, -1.0, 0.0);
        let xyz = v1.xyz();
        let v2 = &Vec3::new(xyz.0, xyz.1, xyz.2);

        assert_eq!(v1, v2);
    }
}
