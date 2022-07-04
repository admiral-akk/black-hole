use web_sys::WebGlTexture;

pub enum TextureStore<'a> {
    ARRAY(&'a [u8]),
    ALLOCATED(&'a WebGlTexture),
}

pub struct Texture<'a> {
    pub store: TextureStore<'a>,
    pub width: i32,
    pub name: String,
}

impl<'b> Texture<'b> {
    pub fn new_from_u8<'a>(arr: &'a [u8], width: i32, name: &'a str) -> Texture<'a> {
        Texture {
            store: TextureStore::ARRAY(arr),
            width,
            name: name.to_string(),
        }
    }
    pub fn new_from_allocated<'a>(tex: &'a WebGlTexture, name: &'a str) -> Texture<'a> {
        Texture {
            store: TextureStore::ALLOCATED(tex),
            width: 0,
            name: name.to_string(),
        }
    }
}
