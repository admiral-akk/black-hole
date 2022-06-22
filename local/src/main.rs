use std::env;
use std::path::Path;

use rendering::{init, structs::dimensions::Dimensions};
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let mut file_name: String = "image.png".to_string();

    if args.len() > 2 {
        file_name = format!("{}.png", args[2]);
    }
    let mut dimensions = Dimensions::new(200, 100);

    if args.len() == 5 {
        dimensions.width = str::parse(&args[3]).unwrap();
        dimensions.height = str::parse(&args[4]).unwrap();
    }

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
