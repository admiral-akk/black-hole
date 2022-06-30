#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_4, PI, TAU};

    use glam::Vec3;
    use rendering::{
        render::render,
        structs::{
            image_data::ImageData, observer::Observer, polar_coordinates::PolarCoordinates,
            ray_cache::RayCache, stars::Stars,
        },
        utils::extensions::ToVec3,
    };

    fn init(
        pos: Vec3,
        facing: Vec3,
        up: Vec3,
        vertical_fov: f32,
    ) -> (ImageData, Observer, Stars, RayCache) {
        let dim = 800;
        let observer = Observer::new(-pos.length() * Vec3::Z, facing, up, vertical_fov);
        let image_data = ImageData::new(dim, dim);

        let background = image::open("uv.png").unwrap();
        let radius = 1.0;

        let ray_cache = RayCache::compute_new(10000, radius, pos.length());
        let mut stars = Stars::new(background);
        stars.update_position(&pos);

        (image_data, observer, stars, ray_cache)
    }

    fn rad_angles(count: u32) -> Vec<f32> {
        let mut arr = Vec::new();
        for i in 0..count {
            arr.push(TAU * (i as f32) / (count as f32));
        }
        arr
    }

    #[test]
    fn base_observer() -> Result<(), Box<dyn std::error::Error>> {
        let pos = -5.0 * Vec3::Z;
        let vertical_fov = 90.0;
        let (mut image_data, observer, stars, black_hole) = init(pos, -pos, Vec3::Y, vertical_fov);
        render(&mut image_data, &observer, &stars, &black_hole);

        let file_name = format!("observer/base_observer");
        image_data.write_image(&file_name);
        Ok(())
    }

    #[test]
    fn rotation_xz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: 0.0,
                phi: rad,
            };
            let pos = -5.0 * polar.to_vec3();
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) =
                init(pos, -pos, Vec3::Y, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_XZ_angle_{:.0}", rad.to_degrees());
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_xy_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: rad,
                phi: 90.0_f32.to_radians(),
            };
            let pos = -5.0 * polar.to_vec3();
            let vertical_fov = 90.0;
            let up = Vec3::Z;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_XY_angle_{:.0}", rad.to_degrees());
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_yz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: rad,
                phi: 0.0,
            };
            let pos = -5.0 * polar.to_vec3();
            let vertical_fov = 90.0;
            let up = Vec3::X;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_YZ_angle_{:.0}", rad.to_degrees());
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_x_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: rad,
                phi: 0.0,
            };
            let pos = -5.0 * Vec3::X;
            let up = -5.0 * polar.to_vec3();
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!(
                "observer/observer_rotate_up_X_angle_{:.0}",
                rad.to_degrees()
            );
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_y_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: 0.0,
                phi: rad,
            };
            let vertical_fov = 90.0;
            let pos = -5.0 * Vec3::Y;
            let up = -5.0 * polar.to_vec3();
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!(
                "observer/observer_rotate_up_Y_angle_{:.0}",
                rad.to_degrees()
            );
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_up_z_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: rad,
                phi: 90.0_f32.to_degrees(),
            };
            let up = -5.0 * polar.to_vec3();
            let pos = -5.0 * Vec3::Z;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, -pos, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!(
                "observer/observer_rotate_up_Z_angle_{:.0}",
                rad.to_degrees()
            );
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_view_xz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: 0.0,
                phi: rad,
            };
            let dir = polar.to_vec3();
            let pos = -5.0 * Vec3::Z;
            let vertical_fov = 90.0;
            let up = Vec3::Y;
            let (mut image_data, observer, stars, ray_cache) = init(pos, dir, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!(
                "observer/observer_rotate_view_XZ_angle_{:.0}",
                rad.to_degrees()
            );
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn rotation_view_yz_observer() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: rad,
                phi: 0.0,
            };
            let dir = polar.to_vec3();
            let pos = -5.0 * Vec3::Z;
            let vertical_fov = 90.0;
            let up = Vec3::X;
            let vertical_fov = 90.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, dir, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!(
                "observer/observer_rotate_view_YZ_angle_{:.0}",
                rad.to_degrees()
            );
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn set_distance_observer() -> Result<(), Box<dyn std::error::Error>> {
        let steps = 20;
        for step in 3..=steps {
            let dist = (step as f32) / 2.0;
            let pos = -dist * Vec3::Z;
            let vertical_fov = 120.0;
            let (mut image_data, observer, stars, ray_cache) =
                init(pos, -pos, Vec3::Y, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_distance_{:.1}", dist);
            image_data.write_image(&file_name);
        }
        Ok(())
    }

    #[test]
    fn orbit_close_to_horizon() -> Result<(), Box<dyn std::error::Error>> {
        let rotation_count = 8;
        for rad in rad_angles(rotation_count) {
            let polar = PolarCoordinates {
                theta: 0.0,
                phi: rad,
            };
            let pos = -1.5 * polar.to_vec3();
            let polar = PolarCoordinates {
                theta: 0.0,
                phi: rad + FRAC_PI_4,
            };
            let target_pos = -1.2 * polar.to_vec3();
            let dir = (target_pos - pos).normalize();
            let up = (Vec3::X - Vec3::X.dot(dir.normalize()) * dir.normalize()).normalize();
            let vertical_fov = 120.0;
            let (mut image_data, observer, stars, ray_cache) = init(pos, dir, up, vertical_fov);
            render(&mut image_data, &observer, &stars, &ray_cache);

            let file_name = format!("observer/observer_orbit_XZ_angle_{:.0}", rad.to_degrees());
            image_data.write_image(&file_name);
        }
        Ok(())
    }
}
