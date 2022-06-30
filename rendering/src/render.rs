use structs::stars::Stars;

use crate::structs::{
    self,
    data::{Data, DEFAULT_DATA},
    image_data::ImageData,
    observer::Observer,
    ray_cache::RayCache,
};

pub fn render(
    image_data: &mut ImageData,
    observer: &Observer,
    stars: &Stars,
    ray_cache: &RayCache,
) {
    // We need to calculate the rgba value of each pixel. We can do this by:
    // 1. iterating over x/y
    // 2. asking the camera to generate a bunch of rays
    // 3. asking the black hole what those rays resolve to
    // 4. recombining the values into a single rgba value.

    // get an array to store the data
    let mut data = vec![DEFAULT_DATA; image_data.get_sample_count()];

    // get the index -> view_port
    image_data.set_samples(&mut data);

    // get the view_port -> start_dir
    observer.to_start_dir(&mut data);

    // get the start_dir -> final_dir
    // get the final_dir -> polar coordinates
    ray_cache.calculate_final_dir(&mut data);

    // get the polar_coordinates -> colors
    stars.to_rgba(&mut data);

    // apply the colors to image
    image_data.load_colors(&data);
}
