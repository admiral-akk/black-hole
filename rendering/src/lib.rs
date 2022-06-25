use renderer::renderer::Renderer;

pub mod render;
pub mod renderer;
pub mod structs;

pub fn init() -> Renderer {
    Renderer::new()
}
