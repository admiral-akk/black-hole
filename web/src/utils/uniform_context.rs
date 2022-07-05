use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlTexture};

use super::{program_context::ProgramContext, render_context::RenderContext};

pub enum UniformStore {
    TEXTURE_2D(WebGlTexture),
    ARRAY_F32(Vec<f32>),
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

    pub fn array_f32(gl: &RenderContext, arr: &[f32], name: &str) -> UniformContext {
        UniformContext {
            store: UniformStore::ARRAY_F32(arr.to_vec()),
            name: name.to_string(),
        }
    }

    pub fn add_to_program(&self, gl: &RenderContext, program: &mut ProgramContext) {
        match &self.store {
            UniformStore::TEXTURE_2D(texture) => {
                add_texture_to_program(&texture, gl, program, &self.name);
            }
            UniformStore::ARRAY_F32(arr) => {
                let loc = gl.gl.get_uniform_location(&program.program, &self.name);
                gl.gl.uniform1fv_with_f32_array(loc.as_ref(), arr);
            }
        }
    }
}

fn add_texture_to_program(
    texture: &WebGlTexture,
    gl: &RenderContext,
    program: &mut ProgramContext,
    name: &str,
) {
    let gl = &gl.gl;
    let texture_count = program.get_and_increment_texture_count();
    gl.active_texture(WebGl2RenderingContext::TEXTURE0 + texture_count);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
    let loc = gl.get_uniform_location(&program.program, name);
    gl.uniform1i(loc.as_ref(), texture_count as i32);
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
