use path_integration::BlackHole;
use structs::{camera::Camera, stars::Stars};

use crate::structs;

pub fn render(camera: &mut Camera, stars: &Stars, black_hole: &BlackHole) {
    // We need to calculate the rgba value of each pixel. We can do this by:
    // 1. iterating over x/y
    // 2. asking the camera to generate a bunch of rays
    // 3. asking the black hole what those rays resolve to
    // 4. recombining the values into a single rgba value.

    for x in 0..camera.get_dimensions().width {
        for y in 0..camera.get_dimensions().height {
            let rays = camera.get_rays(x, y);
            let ray_count = rays.len();
            let mut color = [0, 0, 0, 255];
            for ray in rays {
                let final_dir = black_hole.final_dir(&ray);
                if final_dir.is_some() {
                    let c = stars.get_rgba(&(final_dir.unwrap()));
                    for i in 0..4 {
                        color[i] += c[i] as u32;
                    }
                }
            }
            let c = [
                (color[0] / ray_count as u32) as u8,
                (color[1] / ray_count as u32) as u8,
                (color[2] / ray_count as u32) as u8,
                255,
            ];

            camera.write_color(x, y, &c);
        }
    }
}
