use web_sys::{WebGl2RenderingContext, WebGlTexture};

use super::{program_context::ProgramContext, render_context::RenderContext};

pub enum UniformStore<'a> {
    Texture2dRef(&'a WebGlTexture),
    ArrayF32(Vec<f32>),
}

pub struct UniformContext<'a> {
    pub store: UniformStore<'a>,
    pub name: String,
}

impl UniformContext<'_> {
    pub fn new_from_allocated_ref<'a>(texture: &'a WebGlTexture, name: &str) -> UniformContext<'a> {
        UniformContext {
            store: UniformStore::Texture2dRef(texture),
            name: name.to_string(),
        }
    }

    pub fn array_f32<'a>(arr: &[f32], name: &str) -> UniformContext<'a> {
        UniformContext {
            store: UniformStore::ArrayF32(arr.to_vec()),
            name: name.to_string(),
        }
    }

    pub fn add_to_program(&self, gl: &RenderContext, program: &mut ProgramContext) {
        match &self.store {
            UniformStore::ArrayF32(arr) => {
                let loc = gl.gl.get_uniform_location(&program.program, &self.name);
                gl.gl.uniform1fv_with_f32_array(loc.as_ref(), arr);
            }
            UniformStore::Texture2dRef(texture) => {
                add_texture_to_program(&texture, gl, program, &self.name);
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
