use path_integration::BlackHole;
use structs::stars::Stars;

use crate::structs::{self, image_data::ImageData, observer::Observer};

pub fn render(
    image_data: &mut ImageData,
    observer: &Observer,
    stars: &Stars,
    black_hole: &BlackHole,
) {
    // We need to calculate the rgba value of each pixel. We can do this by:
    // 1. iterating over x/y
    // 2. asking the camera to generate a bunch of rays
    // 3. asking the black hole what those rays resolve to
    // 4. recombining the values into a single rgba value.

    let (width, height) = image_data.get_dimensions();

    for x in 0..width {
        for y in 0..height {
            let samples = image_data.get_samples(x, y);
            let rays = observer.to_rays(&samples);
            for ray in rays {
                let final_dir = black_hole.final_dir(&ray);
                let mut c = [0, 0, 0, 255];
                if final_dir.is_some() {
                    c = stars.get_rgba(&(final_dir.unwrap()));
                }
                image_data.add_sample(x, y, &c);
            }
        }
    }
}
