use std::{
    env,
    f32::consts::{FRAC_PI_4, TAU},
    fs,
    path::Path,
};

use glam::Vec3;

use image::io::Reader;

use rendering::{
    render::render,
    structs::{
        dimensions::Dimensions, image_data::ImageData, observer::Observer, ray_cache::RayCache,
        stars::Stars,
    },
};

fn circular_orbit(distance: f32, count: usize) -> Vec<(Vec3, Vec3)> {
    let mut pos_dir = Vec::new();
    for i in 0..count {
        let angle = TAU * (i as f32) / (count as f32);
        let pos = -distance * (angle.cos() * Vec3::Z + angle.sin() * Vec3::X);
        pos_dir.push((pos, -pos));
    }
    pos_dir
}

fn circular_orbit_facing_horizon(distance: f32, count: usize) -> Vec<(Vec3, Vec3)> {
    let mut pos_dir = Vec::new();
    for i in 0..count {
        let angle = TAU * (i as f32) / (count as f32);
        let pos = -distance * (angle.cos() * Vec3::Z + angle.sin() * Vec3::X);
        let horizon_angle = angle + FRAC_PI_4;
        let dir = -distance * (horizon_angle.cos() * Vec3::Z + horizon_angle.sin() * Vec3::X) - pos;
        pos_dir.push((pos, dir));
    }
    pos_dir
}

fn main() {
    let mut file_name: String = "image".to_string();
    let mut dimensions = Dimensions::new(100, 100);

    set_up(&mut file_name, &mut dimensions);

    let distance = 3.0;
    let vertical_fov = 120.0;

    let mut reader = Reader::open("starmap_2020_4k_gal.exr").unwrap();
    reader.no_limits();
    let background = reader.decode().unwrap().into_rgb8();
    let radius = 1.5;

    let mut image_data = ImageData::new(dimensions.width, dimensions.height);
    let mut stars = Stars::new(image::DynamicImage::ImageRgb8(background));
    let ray_cache = RayCache::compute_new(1024, radius, distance);

    let folder_name = format!("main/{}", file_name);
    let full_folder_name = format!("output/{}", &folder_name);
    if Path::new(&full_folder_name).exists() {
        fs::remove_dir_all(&full_folder_name).unwrap();
    }
    fs::create_dir(&full_folder_name).unwrap();

    let orbit = circular_orbit(distance as f32, 100);
    for i in 0..orbit.len() {
        let (pos, dir) = orbit[i];
        let observer = Observer::new(pos, dir, Vec3::Y, vertical_fov);
        stars.update_position(&pos);
        render(&mut image_data, &observer, &stars, &ray_cache);
        let frame_name = format!("{}/frame_{:04}", folder_name, i);

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
