use std::env;
use std::path::Path;

use glam::DVec3;

use image::io::Reader;
use path_integration::BlackHole;
use rendering::{
    render::render,
    structs::{camera::Camera, dimensions::Dimensions, stars::Stars},
};
fn main() {
    let mut file_name: String = "image.png".to_string();
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

    let mut camera = Camera::new(dimensions, pos, vertical_fov);
    let black_hole = BlackHole::new(radius, &pos, std::f64::consts::PI * vertical_fov / 180.0);
    let stars = Stars::new(image::DynamicImage::ImageRgba32F(background));
    let (width, height) = camera.get_dimensions();
    render(&mut camera, &stars, &black_hole);

    image::save_buffer(
        &Path::new(&format!("output/{}", file_name)),
        camera.get_colors(),
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}

fn set_up(file_name: &mut String, dimensions: &mut Dimensions) {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() > 1 {
        *file_name = format!("{}.png", args[1]);
    }
    if args.len() == 4 {
        dimensions.width = str::parse(&args[2]).unwrap();
        dimensions.height = str::parse(&args[3]).unwrap();
    }
}
