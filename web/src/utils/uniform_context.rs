use web_sys::WebGlTexture;

pub enum TextureStore<'a> {
    ARRAY(&'a [u8]),
    ALLOCATED(&'a WebGlTexture),
}

pub struct UniformContext<'a> {
    pub store: TextureStore<'a>,
    pub width: i32,
    pub name: String,
}

impl<'b> UniformContext<'b> {
    pub fn new_from_u8<'a>(arr: &'a [u8], width: i32, name: &'a str) -> UniformContext<'a> {
        UniformContext {
            store: TextureStore::ARRAY(arr),
            width,
            name: name.to_string(),
        }
    }
    pub fn new_from_allocated<'a>(tex: &'a WebGlTexture, name: &'a str) -> UniformContext<'a> {
        UniformContext {
            store: TextureStore::ALLOCATED(tex),
            width: 0,
            name: name.to_string(),
        }
    }
}
