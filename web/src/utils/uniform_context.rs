use web_sys::{WebGl2RenderingContext, WebGlTexture};

use super::render_context::RenderContext;

pub enum UniformStore {
    TEXTURE_2D(WebGlTexture),
}

pub struct UniformContext {
    pub store: UniformStore,
    pub name: String,
}

impl UniformContext {
    pub fn new_from_u8(gl: &RenderContext, arr: &[u8], width: i32, name: &str) -> UniformContext {
        let texture = generate_texture_from_u8(&gl.gl, arr, width);
        UniformContext {
            store: UniformStore::TEXTURE_2D(texture),
            name: name.to_string(),
        }
    }
    pub fn new_from_allocated(texture: WebGlTexture, name: &str) -> UniformContext {
        UniformContext {
            store: UniformStore::TEXTURE_2D(texture),
            name: name.to_string(),
        }
    }
}

fn generate_texture_from_u8(gl: &WebGl2RenderingContext, arr: &[u8], width: i32) -> WebGlTexture {
    let texture = gl.create_texture();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGBA as i32,
        width,
        (arr.len() / (4 * width) as usize) as i32,
        0,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        Some(arr),
    )
    .unwrap();
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_S,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_T,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    texture.unwrap()
}
