use glam::DVec3;
#[derive(Debug, Clone)]
pub struct Ray {
    pub pos: DVec3,
    pub dir: DVec3,
}

impl Ray {
    pub fn new(pos: DVec3, dir: DVec3) -> Self {
        Self {
            pos,
            dir: dir.normalize(),
        }
    }
}

#[cfg(test)]

mod tests {
    use glam::DVec3;

    use crate::Ray;

    #[test]
    fn ray_dir_returns_vec3() {
        let v = DVec3::new(1.0, -2.0, 3.0);
        let unit_v = DVec3::new(1.0, 1.0, 1.0);
        let ray = Ray::new(v, unit_v);

        assert_eq!(DVec3::new(1.0, 1.0, 1.0).normalize(), ray.dir);
    }
}
