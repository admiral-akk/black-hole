#[cfg(test)]

mod tests {
    use glam::DVec3;
    use path_integration::Ray;

    #[test]
    fn ray_dir_returns_vec3() {
        let v = DVec3::new(1.0, -2.0, 3.0);
        let unit_v = DVec3::new(1.0, 1.0, 1.0);
        let ray = Ray::new(v, unit_v);

        assert_eq!(DVec3::new(1.0, 1.0, 1.0).normalize(), ray.dir);
    }
}
