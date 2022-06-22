use renderer::Renderer;
use structs::{config::Config};

pub mod renderer;
pub mod structs;

pub fn init() -> Renderer {
    let config = Config::new();
    Renderer::new(config)
}
