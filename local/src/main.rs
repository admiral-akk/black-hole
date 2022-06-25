use std::env;
use std::path::Path;

use geometry::{DVec3, Vec3};

use rendering::{
    init,
    render::render,
    renderer::renderer::{RenderConfig, RenderType},
    structs::{black_hole::BlackHole, camera::Camera, dimensions::Dimensions, stars::Stars},
};

fn main() {
    let mut file_name: String = "image.png".to_string();
    let mut dimensions = Dimensions::new(100, 100);

    set_up(&mut file_name, &mut dimensions);

    let pos = DVec3::ZERO;
    let dir = DVec3::Z;
    let vertical_fov = 50.0;

    let background = image::open("starmap_2020_4k_gal.exr").unwrap();

    let black_hole_pos = 5.0 * DVec3::Z;
    let radius = 1.0;

    let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);
    let black_hole = BlackHole::new(black_hole_pos, radius);
    let stars = Stars::new(background);
    render(&mut camera, &stars, &black_hole);

    image::save_buffer(
        &Path::new(&format!("output/{}", file_name)),
        camera.get_colors(),
        camera.get_dimensions().width as u32,
        camera.get_dimensions().height as u32,
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
