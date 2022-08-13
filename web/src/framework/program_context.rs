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
        if compilation_log.is_some() {
            let compilation_log = compilation_log.unwrap();
            if compilation_log.contains("ERROR") || compilation_log.contains("WARNING") {
                let source: Vec<String> = fragment_source
                    .generate_source()
                    .split("\n")
                    .map(|s| s.to_string())
                    .collect();
                let log_vec: Vec<&str> = compilation_log.split("\n").collect();

                for i in 0..log_vec.len() {
                    let line = log_vec[i];
                    if !line.contains("ERROR") && !line.contains("WARNING") {
                        continue;
                    }
                    let split_line: Vec<&str> = line.split(":").collect();
                    let line_number = split_line[2].parse::<usize>();
                    if !line_number.is_ok() {
                        continue;
                    }
                    let line_number = line_number.unwrap();
                    let source_line = &source[line_number - 1];
                    console_log!("Log line: {:?}\nSource line: {}", line, source_line);
                }
            }
        }
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
