use std::env;
use std::path::Path;

use geometry::Vec3;

use rendering::{
    init,
    renderer::renderer::{RenderConfig, RenderType},
    structs::dimensions::Dimensions,
};

fn main() {
    let mut file_name: String = "image.png".to_string();
    let mut dimensions = Dimensions::new(200, 100);

    set_up(&mut file_name, &mut dimensions);
    let img = image::open("space-background.jpg").unwrap();

    let config = RenderConfig {
        dimensions,
        render_type: RenderType::BlackSphere {
            background: img,
            vertical_fov_degrees: 80.0,
            pos: Vec3::new(0.0, 0.0, 10.0),
            rad: 9.0,
        },
    };
    render_image(file_name, config);
}

fn set_up(file_name: &mut String, dimensions: &mut Dimensions) {
    let args: Vec<String> = env::args().collect();
    print!("{:?}", args);
    if args.len() > 1 {
        *file_name = format!("{}.png", args[1]);
    }
    if args.len() == 4 {
        dimensions.width = str::parse(&args[2]).unwrap();
        dimensions.height = str::parse(&args[3]).unwrap();
    }
}

fn render_image(file_name: String, config: RenderConfig) {
    let renderer = init();
    let mut buffer: Vec<u8> = config.dimensions.get_buffer();
    renderer.render(&mut buffer, &config);
    image::save_buffer(
        &Path::new(&format!("output/{}", file_name)),
        &buffer,
        config.dimensions.width as u32,
        config.dimensions.height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
