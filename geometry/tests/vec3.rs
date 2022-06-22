#[cfg(test)]

mod tests {
    use geometry::Vec3;

    #[test]
    fn vec3_mul() {
        let v = Vec3::new(1.0, -2.0, 3.0);

        assert_eq!(2.0 * &v, Vec3::new(2.0, -4.0, 6.0));
        assert_eq!(-2.0 * &v, Vec3::new(-2.0, 4.0, -6.0));
        assert_eq!(0.0 * &v, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(&v * 2.0, Vec3::new(2.0, -4.0, 6.0));
        assert_eq!(&v * -2.0, Vec3::new(-2.0, 4.0, -6.0));
        assert_eq!(&v * 0.0, Vec3::new(0.0, 0.0, 0.0));
    }
}
