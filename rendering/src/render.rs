use path_integration::RayCache;
use structs::{black_hole::BlackHole, camera::Camera, stars::Stars};

use crate::structs;

pub fn render(camera: &mut Camera, stars: &Stars, black_hole: &BlackHole) {
    let rays = camera.get_rays();
    let cache = RayCache::compute_new(
        10000,
        &black_hole.field,
        &camera.pos,
        std::f64::consts::PI * camera.vertical_fov / 180.0,
    );
    for i in 0..rays.len() {
        let final_dir = cache.final_dir(&rays[i], &black_hole.field);
        //let final_dir = black_hole.raycast(ray);
        if final_dir.is_none() {
            let c = [0u8, 0u8, 0u8, 255u8];
            camera.write_color(i, &c);
        } else {
            let c = stars.get_rgba(&(final_dir.unwrap()));
            camera.write_color(i, &c);
        }
    }
}
