use crate::structs::config::Config;

pub struct Renderer {
    config: Config,
}

impl Renderer {
    pub fn new(config: Config) -> Renderer {
        Renderer { config }
    }
}
