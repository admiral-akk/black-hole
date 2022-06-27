use std::env;

use glam::DVec3;

use image::io::Reader;
use path_integration::BlackHole;
use rendering::{
    render::render,
    structs::{dimensions::Dimensions, image_data::ImageData, observer::Observer, stars::Stars},
};
fn main() {
    let mut file_name: String = "image".to_string();
    let mut dimensions = Dimensions::new(100, 100);

    set_up(&mut file_name, &mut dimensions);

    let pos = -5.0 * DVec3::Z;
    let vertical_fov = 60.0;

    let background = Reader::open("milkyway_2020_4k_gal.exr")
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba32f();
    let radius = 1.5;

    let observer = Observer::new(pos, DVec3::Y, vertical_fov);
    let mut image_data = ImageData::new(dimensions.width, dimensions.height);
    let black_hole = BlackHole::new(
        radius,
        pos.length(),
        std::f64::consts::PI * vertical_fov / 180.0,
    );
    let stars = Stars::new(image::DynamicImage::ImageRgba32F(background));
    render(&mut image_data, &observer, &stars, &black_hole);
    image_data.write_image(&file_name);
}

fn set_up(file_name: &mut String, dimensions: &mut Dimensions) {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() > 1 {
        *file_name = format!("{}", args[1]);
    }
    if args.len() == 4 {
        dimensions.width = str::parse(&args[2]).unwrap();
        dimensions.height = str::parse(&args[3]).unwrap();
    }
}
