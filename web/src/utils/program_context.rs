use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlTexture};

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

impl ProgramContext {
    pub fn new(
        gl: &WebGl2RenderingContext,
        vertex_source: &str,
        fragment_source: &str,
    ) -> ProgramContext {
        let shader = gl
            .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
            .ok_or_else(|| String::from("Unable to create shader object"))
            .unwrap();
        gl.shader_source(&shader, vertex_source);
        gl.compile_shader(&shader);

        let vert_shader = shader;

        let shader = gl
            .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
            .ok_or_else(|| String::from("Unable to create shader object"))
            .unwrap();
        gl.shader_source(&shader, fragment_source);
        gl.compile_shader(&shader);

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

    pub fn add_texture(&mut self, gl: &WebGl2RenderingContext, texture: &WebGlTexture, name: &str) {
        let program = &self.program;
        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + self.texture_count);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
        let loc = gl.get_uniform_location(program, name);
        gl.uniform1i(loc.as_ref(), self.texture_count as i32);
        self.texture_count += 1;
    }

    pub fn add_texture_from_u8(
        &mut self,
        gl: &WebGl2RenderingContext,
        arr: &[u8],
        width: i32,
        name: &str,
    ) {
        let program = &self.program;
        let texture = gl.create_texture();
        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + self.texture_count);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
        let loc = gl.get_uniform_location(program, name);
        gl.uniform1i(loc.as_ref(), self.texture_count as i32);

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
        self.texture_count += 1;
    }
}
