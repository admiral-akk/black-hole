use std::{env, f64::consts::TAU};

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

    let distance = 5.0;
    let vertical_fov = 60.0;

    let mut reader = Reader::open("milkyway_2020_4k_gal.exr").unwrap();
    reader.no_limits();
    let background = reader.decode().unwrap().into_rgb8();
    let radius = 1.5;

    let mut image_data = ImageData::new(dimensions.width, dimensions.height);
    let stars = Stars::new(image::DynamicImage::ImageRgb8(background));
    let black_hole = BlackHole::new(radius, distance);
    let iterations = 30;

    for i in 0..iterations {
        let angle = TAU * (i as f64) / (iterations as f64);
        let pos = -distance * (angle.cos() * DVec3::Z + angle.sin() * DVec3::X);
        let observer = Observer::new(pos, -pos, DVec3::Y, vertical_fov);
        render(&mut image_data, &observer, &stars, &black_hole);
        let frame_name = format!("animation/clear_4/{}_frame_{:04}", file_name, i);

        image_data.write_image(&frame_name);
    }
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
