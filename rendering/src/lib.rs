use renderer::Renderer;

pub mod renderer;
pub mod structs;

pub fn init() -> Renderer {
    Renderer::new()
}
