#[cfg(test)]

mod tests {
    use geometry::{Ray, Sphere, Vec3};

    #[test]
    fn sphere_hit() {
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 10.0), 1.0);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(sphere.is_hit(&ray), true);
    }

    #[test]
    fn sphere_graze() {
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 10.0), 1.0);
        let ray = Ray::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(sphere.is_hit(&ray), true);
    }

    #[test]
    fn sphere_miss() {
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 10.0), 1.0);
        let ray = Ray::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(sphere.is_hit(&ray), false);
    }
}
