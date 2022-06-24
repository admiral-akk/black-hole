#[cfg(test)]

mod tests {
    use geometry::{Disc, Ray, Vec3};

    #[test]
    fn disc_miss_inside() {
        let disc = Disc::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 1.0),
            2.0,
            1.0,
        );
        let ray = &Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(disc.is_hit(ray), false);
    }

    #[test]
    fn disc_graze_inside() {
        let disc = Disc::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 1.0),
            2.0,
            1.0,
        );
        let ray = &Ray::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(disc.is_hit(ray), true);
    }
    #[test]
    fn disc_hit() {
        let disc = Disc::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 1.0),
            2.0,
            1.0,
        );
        let ray = &Ray::new(Vec3::new(1.5, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(disc.is_hit(ray), true);
    }
    #[test]
    fn disc_graze_outside() {
        let disc = Disc::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 1.0),
            2.0,
            1.0,
        );
        let ray = &Ray::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(disc.is_hit(ray), true);
    }
    #[test]
    fn disc_miss_outside() {
        let disc = Disc::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 1.0),
            2.0,
            1.0,
        );
        let ray = &Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(disc.is_hit(ray), false);
    }
}
