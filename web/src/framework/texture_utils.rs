use web_sys::{WebGl2RenderingContext, WebGlTexture};

pub enum Format {
    R,
    RG,
    RGB,
    RGBA,
}

impl Format {
    pub fn dimension(&self) -> i32 {
        match self {
            Format::R => 1,
            Format::RG => 2,
            Format::RGB => 3,
            Format::RGBA => 4,
        }
    }
    pub fn external_format(&self) -> u32 {
        match self {
            Format::R => WebGl2RenderingContext::RED,
            Format::RG => WebGl2RenderingContext::RG,
            Format::RGB => WebGl2RenderingContext::RGB,
            Format::RGBA => WebGl2RenderingContext::RGBA,
        }
    }
}

fn add_default_tex_parameters(gl: &WebGl2RenderingContext) {
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
}

fn add_default_3d_tex_parameters(gl: &WebGl2RenderingContext) {
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_3D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_3D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_3D,
        WebGl2RenderingContext::TEXTURE_WRAP_S,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_3D,
        WebGl2RenderingContext::TEXTURE_WRAP_T,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_3D,
        WebGl2RenderingContext::TEXTURE_WRAP_R,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
}

pub fn create_texture(
    gl: &WebGl2RenderingContext,
    width: i32,
    height: i32,
) -> Option<WebGlTexture> {
    let tex = gl.create_texture();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, tex.as_ref());
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGBA32F as i32,
        width,
        height,
        0,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::FLOAT,
        None,
    )
    .expect("Failed to generate texture for frame buffer");
    add_default_tex_parameters(gl);
    tex
}

pub fn generate_texture_from_f32(
    gl: &WebGl2RenderingContext,
    arr: &[f32],
    width: i32,
    format: Format,
) -> WebGlTexture {
    let internal_format = match format {
        Format::R => WebGl2RenderingContext::R32F,
        Format::RG => WebGl2RenderingContext::RG32F,
        Format::RGB => WebGl2RenderingContext::RGB32F,
        Format::RGBA => WebGl2RenderingContext::RGBA32F,
    };
    let texture = gl.create_texture();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
    let buffer = gl.create_buffer().ok_or("Failed to create buffer").unwrap();
    gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(&buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        // Safe as long as there's no memory allocation between this and buffering the data to webgl.
        let positions_array_buf_view = js_sys::Float32Array::view(&arr);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            internal_format as i32,
            width,
            (arr.len() / (format.dimension() * width) as usize) as i32,
            0,
            format.external_format(),
            WebGl2RenderingContext::FLOAT,
            Some(&buffer),
        )
        .unwrap();
    }

    add_default_tex_parameters(gl);
    gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, None);
    texture.unwrap()
}

pub fn generate_3d_texture_from_f32(
    gl: &WebGl2RenderingContext,
    arr: &[f32],
    width: i32,
    height: i32,
    depth: i32,
    format: Format,
) -> WebGlTexture {
    let internal_format = match format {
        Format::R => WebGl2RenderingContext::R32F,
        Format::RG => WebGl2RenderingContext::RG32F,
        Format::RGB => WebGl2RenderingContext::RGB32F,
        Format::RGBA => WebGl2RenderingContext::RGBA32F,
    };
    let texture = gl.create_texture();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_3D, texture.as_ref());
    let buffer = gl.create_buffer().ok_or("Failed to create buffer").unwrap();
    gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(&buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        // Safe as long as there's no memory allocation between this and buffering the data to webgl.
        let positions_array_buf_view = js_sys::Float32Array::view(&arr);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        gl.tex_image_3d_with_opt_array_buffer_view(
            WebGl2RenderingContext::TEXTURE_3D,
            0,
            internal_format as i32,
            depth,
            height,
            width,
            0,
            format.external_format(),
            WebGl2RenderingContext::FLOAT,
            Some(&buffer),
        )
        .unwrap();
    }

    add_default_3d_tex_parameters(gl);
    gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, None);
    texture.unwrap()
}

pub fn generate_texture_from_u8(
    gl: &WebGl2RenderingContext,
    arr: &[u8],
    width: i32,
    format: Format,
) -> WebGlTexture {
    let texture = gl.create_texture();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
    let internal_format = match format {
        Format::R => WebGl2RenderingContext::R8,
        Format::RG => WebGl2RenderingContext::RG8,
        Format::RGB => WebGl2RenderingContext::RGB8,
        Format::RGBA => WebGl2RenderingContext::RGBA8,
    };
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        internal_format as i32,
        width,
        (arr.len() / (format.dimension() * width) as usize) as i32,
        0,
        format.external_format(),
        WebGl2RenderingContext::UNSIGNED_BYTE,
        Some(arr),
    )
    .unwrap();
    add_default_tex_parameters(gl);
    texture.unwrap()
}
