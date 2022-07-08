use glam::{IVec2, Vec2, Vec3, Vec4};
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use super::{program_context::ProgramContext, render_context::RenderContext};

pub enum UniformStore<'a> {
    I32(i32),
    F32(f32),
    Vec2(Vec2),
    IVec2(IVec2),
    Vec3(Vec3),
    Vec4(Vec4),
    ArrayF32(Vec<f32>),
    Texture2dRef(&'a WebGlTexture),
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
    pub fn i32<'a>(i: i32, name: &str) -> UniformContext<'a> {
        UniformContext {
            store: UniformStore::I32(i),
            name: name.to_string(),
        }
    }
    pub fn ivec2<'a>(vec: IVec2, name: &str) -> UniformContext<'a> {
        UniformContext {
            store: UniformStore::IVec2(vec),
            name: name.to_string(),
        }
    }
    pub fn f32<'a>(f: f32, name: &str) -> UniformContext<'a> {
        UniformContext {
            store: UniformStore::F32(f),
            name: name.to_string(),
        }
    }
    pub fn vec2<'a>(vec: Vec2, name: &str) -> UniformContext<'a> {
        UniformContext {
            store: UniformStore::Vec2(vec),
            name: name.to_string(),
        }
    }

    pub fn vec3<'a>(vec: Vec3, name: &str) -> UniformContext<'a> {
        UniformContext {
            store: UniformStore::Vec3(vec),
            name: name.to_string(),
        }
    }

    pub fn add_to_program(&self, gl: &RenderContext, program: &mut ProgramContext) {
        match &self.store {
            UniformStore::I32(i) => {
                let loc = gl.gl.get_uniform_location(&program.program, &self.name);
                gl.gl.uniform1i(loc.as_ref(), *i);
            }
            UniformStore::IVec2(v) => {
                let loc = gl.gl.get_uniform_location(&program.program, &self.name);
                gl.gl.uniform2i(loc.as_ref(), v.x, v.y);
            }
            UniformStore::F32(f) => {
                let loc = gl.gl.get_uniform_location(&program.program, &self.name);
                gl.gl.uniform1f(loc.as_ref(), *f);
            }
            UniformStore::Vec2(v) => {
                let loc = gl.gl.get_uniform_location(&program.program, &self.name);
                gl.gl.uniform2f(loc.as_ref(), v.x, v.y);
            }
            UniformStore::Vec3(v) => {
                let loc = gl.gl.get_uniform_location(&program.program, &self.name);
                gl.gl.uniform3f(loc.as_ref(), v.x, v.y, v.z);
            }
            UniformStore::Vec4(v) => {
                let loc = gl.gl.get_uniform_location(&program.program, &self.name);
                gl.gl.uniform4f(loc.as_ref(), v.x, v.y, v.z, v.w);
            }
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
