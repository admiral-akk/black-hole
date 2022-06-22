use renderer::Renderer;
use structs::{
    config::{Config},
    dimensions::{Dimensions},
};

pub mod renderer;
pub mod structs;

pub fn entry() {
    println!("Hello, world!");
}

pub fn init(dimensions: Dimensions) -> Renderer {
    let config = Config::new(dimensions);
    Renderer::new(config)
}
