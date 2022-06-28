use path_integration::BlackHole;
use structs::stars::Stars;

use crate::structs::{
    self, data::Data, image_data::ImageData, observer::Observer,
    polar_coordinates::PolarCoordinates,
};

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

    // get the index -> view_port
    let mut data = image_data.get_data_buffer();

    // get the view_port -> start_dir
    observer.to_start_dir(&mut data);

    // get the start_dir -> final_dir
    observer.all_to_final_dir(&black_hole, &mut data);

    // get the final_dir -> polar coordinates
    PolarCoordinates::to_polar(&mut data);

    // get the polar_coordinates -> colors
    stars.to_rgba(&mut data);

    // apply the colors to image
    image_data.load_colors();
}
