use renderer::Renderer;
use structs::{config::Config, dimensions::Dimensions};

pub mod renderer;
pub mod structs;

pub fn init() -> Renderer {
    let config = Config::new();
    Renderer::new(config)
}
