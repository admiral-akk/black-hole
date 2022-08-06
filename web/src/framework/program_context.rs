use web_sys::{WebGl2RenderingContext, WebGlProgram};

use crate::console_log;

use super::source_context::SourceContext;

pub struct ProgramContext {
    pub program: WebGlProgram,
    texture_count: u32,
}

fn initalize_position(gl: &WebGl2RenderingContext, program: &WebGlProgram) {
    let position_attribute_location = gl.get_attrib_location(program, "position");

    gl.vertex_attrib_pointer_with_i32(
        position_attribute_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(position_attribute_location as u32);
}

const VERTEX_DEFAULT: &str = include_str!("position.glsl");

impl ProgramContext {
    pub fn new(
        gl: &WebGl2RenderingContext,
        vertex_source: Option<&SourceContext>,
        fragment_source: &SourceContext,
    ) -> ProgramContext {
        let shader = gl
            .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
            .ok_or_else(|| String::from("Unable to create shader object"))
            .unwrap();
        if vertex_source.is_some() {
            let vertex_source = vertex_source.unwrap();
            gl.shader_source(&shader, &vertex_source.generate_source());
        } else {
            gl.shader_source(&shader, VERTEX_DEFAULT);
        }
        gl.compile_shader(&shader);

        let vert_shader = shader;

        let shader = gl
            .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
            .ok_or_else(|| String::from("Unable to create shader object"))
            .unwrap();
        gl.shader_source(&shader, &fragment_source.generate_source());
        gl.compile_shader(&shader);
        let compilation_log = gl.get_shader_info_log(&shader);
        console_log!("Shader log: {:?}", compilation_log);

        let frag_shader = shader;
        let program = gl
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))
            .unwrap();

        gl.attach_shader(&program, &vert_shader);
        gl.attach_shader(&program, &frag_shader);
        gl.link_program(&program);
        initalize_position(gl, &program);
        ProgramContext {
            program,
            texture_count: 0,
        }
    }

    pub fn get_and_increment_texture_count(&mut self) -> u32 {
        let texture_count = self.texture_count;
        self.texture_count += 1;
        texture_count
    }
}
