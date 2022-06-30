#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_4, PI};

    use glam::{DVec3, Vec3};
    use path_integration::Field;
    use rendering::{
        render::render,
        structs::{
            image_data::ImageData, observer::Observer, polar_coordinates::Polar,
            ray_cache::RayCache, stars::Stars,
        },
    };

    fn init(
        pos: DVec3,
        facing: DVec3,
        up: DVec3,
        vertical_fov: f64,
    ) -> (ImageData, Observer, Stars, RayCache) {
        let dim = 800;
        let observer = Observer::new(-pos.length() * DVec3::Z, facing, DVec3::Y, vertical_fov);
        let image_data = ImageData::new(dim, dim);

        let background = image::open("uv.png").unwrap();
        let radius = 1.0;

        let field = Field::new(radius, &pos);
        let ray_cache = RayCache::compute_new(10000, &field, pos.length());
        let mut stars = Stars::new(background);
        let pos2 = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
        stars.update_position(&pos2);

        (image_data, observer, stars, ray_cache)
    }

    #[test]
    fn base_observer() -> Result<(), Box<dyn std::error::Error>> {
        let pos = -5.0 * DVec3::Z;
        let vertical_fov = 90.0;
        let (mut image_data, observer, stars, black_hole) = init(pos, -pos, DVec3::Y, vertical_fov);
        render(&mut image_data, &observer, &stars, &black_hole);

        let file_name = format!("observer/base_observer");
        image_data.write_image(&file_name);
        Ok(())
    }

    #[test]
    fn rotation_xz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * (f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::X);
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) =
                init(pos, -pos, DVec3::Y, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_XZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_xy_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * (f64::cos(angle_rad) * DVec3::Y + f64::sin(angle_rad) * DVec3::X);
            let up = DVec3::Z;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_XY_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_yz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * (f64::cos(angle_rad) * DVec3::Y + f64::sin(angle_rad) * DVec3::Z);
            let up = DVec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_YZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_x_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::X;
            let up = f64::cos(angle_rad) * DVec3::Y + f64::sin(angle_rad) * DVec3::Z;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_rotate_up_X_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_y_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::Y;
            let up = f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_rotate_up_Y_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_z_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::Z;
            let up = f64::cos(angle_rad) * DVec3::Y + f64::sin(angle_rad) * DVec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_rotate_up_Z_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_view_xz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::Z;
            let dir = f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::X;
            let up = DVec3::Y;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, dir, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_rotate_view_XZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_view_yz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -5.0 * DVec3::Z;
            let dir = f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::Y;
            let up = DVec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, dir, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_rotate_view_YZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn set_distance_observer() -> Result<(), Box<dyn std::error::Error>> {
        let steps = 20;
        for step in 3..=steps {
            let dist = (step as f64) / 2.0;
            let pos = -dist * DVec3::Z;
            let vertical_fov = 120.0;
            let (mut image_data, observer, stars, ray_cache) =
                init(pos, -pos, DVec3::Y, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_distance_{:.1}", dist);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn orbit_close_to_horizon() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rot in 0..rotation_count {
            let angle_degrees = 360.0 * (rot as f64) / (rotation_count as f64);
            let angle_rad = angle_degrees * PI / 180.0;
            let pos = -1.5 * (f64::cos(angle_rad) * DVec3::Z + f64::sin(angle_rad) * DVec3::X);
            let target_pos = -1.2
                * (f64::cos(angle_rad + FRAC_PI_4) * DVec3::Z
                    + f64::sin(angle_rad + FRAC_PI_4) * DVec3::X);
            let dir = (target_pos - pos).normalize();
            let up = (DVec3::X - DVec3::X.dot(dir.normalize()) * dir.normalize()).normalize();
            let vertical_fov = 120.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, dir, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_orbit_XZ_angle_{}", angle_degrees);
            image_data.write_image(&file_name);
        }
        Ok(())
    }
}
