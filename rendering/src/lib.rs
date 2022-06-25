use renderer::renderer::Renderer;
use structs::{black_hole::BlackHole, camera::Camera, stars::Stars};

pub mod renderer;
pub mod structs;

pub fn init() -> Renderer {
    Renderer::new()
}

pub fn render(camera: &mut Camera, stars: &Stars, black_hole: &BlackHole) {
    let rays = camera.get_rays();
    for i in 0..rays.len() {
        let ray = &rays[i];
        let final_dir = black_hole.raycast(ray);
        if final_dir.is_none() {
            let c = [0u8; 4];
            camera.write_color(i, &c);
        } else {
            let c = stars.get_rgba(&(final_dir.unwrap()));
            camera.write_color(i, &c);
        }
    }
}
