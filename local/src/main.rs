use std::env;
use std::path::Path;

use rendering::{init, structs::dimensions::Dimensions};
fn main() {
    let mut file_name: String = "image.png".to_string();
    let mut dimensions = Dimensions::new(200, 100);

    set_up(&mut file_name, &mut dimensions);
    render_image(file_name, dimensions);
}

fn set_up(file_name: &mut String, dimensions: &mut Dimensions) {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        *file_name = format!("{}.png", args[2]);
    }
    if args.len() == 5 {
        dimensions.width = str::parse(&args[3]).unwrap();
        dimensions.height = str::parse(&args[4]).unwrap();
    }
}

fn render_image(file_name: String, dimensions: Dimensions) {
    let renderer = init(dimensions.clone());
    let mut buffer: Vec<u8> = vec![0; 4 * dimensions.size()];
    renderer.render(&mut buffer, &dimensions);
    image::save_buffer(
        &Path::new(&format!("output/{}", file_name)),
        &buffer,
        dimensions.width as u32,
        dimensions.height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
