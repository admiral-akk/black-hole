use glam::Vec3;
use rendering::{
    structs::{
        data::Data, image_data::ImageData, observer::Observer, ray_cache::RayCache, stars::Stars,
    },
    utils::extensions::ToPolar,
};

use crate::{
    console_log,
    framework::{
        frame_buffer_context::FrameBufferContext,
        render_context::RenderContext,
        source_context::SourceContext,
        texture_utils::{generate_texture_from_f32, generate_texture_from_u8},
        uniform_context::UniformContext,
    },
    generate_uv, BlackHoleParams,
};

const RENDER_TEXTURE_DEFAULT: &str = include_str!("shaders/fragment/render_texture.glsl");

fn complete(gl: &RenderContext, params: &BlackHoleParams) {
    let uniforms = params.uniform_context();
    let mut text: Vec<&UniformContext> = uniforms.iter().map(|u| u).collect();

    let fb2 = gl.create_framebuffer();
    // get the view_port -> start_dir

    let ray_cache = RayCache::compute_new(
        params.cache_width as usize,
        params.black_hole_radius,
        params.distance,
    );

    let final_dirs: Vec<Vec3> = ray_cache.cache.iter().map(|r| r.final_dir).collect();
    let mut f32_vec: Vec<f32> = Vec::new();
    for i in 0..final_dirs.len() {
        let final_dir = final_dirs[i];
        f32_vec.push(final_dir.x);
        f32_vec.push(final_dir.y);
        f32_vec.push(final_dir.z);
        f32_vec.push(1.0);
    }

    let ray_cache_tex = generate_texture_from_f32(&gl.gl, &f32_vec, final_dirs.len() as i32);
    let ray_context = UniformContext::new_from_allocated_ref(&ray_cache_tex, "ray_cache_tex");
    let ray_length = UniformContext::f32(final_dirs.len() as f32, "ray_cache_length");
    let max_z = UniformContext::f32(ray_cache.max_z, "max_z");
    let fb_context2 = UniformContext::new_from_allocated_ref(&fb2.backing_texture, "start_ray_tex");
    text.push(&ray_context);
    text.push(&ray_length);
    text.push(&max_z);
    text.push(&fb_context2);

    let frag = SourceContext::new(include_str!("shaders/fragment/black_hole/complete.glsl"));
    gl.draw(None, &frag, &text, None);
}

pub fn exercise_10(gl: &RenderContext, params: &BlackHoleParams) {
    complete(gl, params);
    // need:

    //

    // Try this to load imagE: https://stackoverflow.com/questions/70309403/updating-html-canvas-imagedata-using-rust-webassembly
    // or this: https://users.rust-lang.org/t/a-future-for-loading-images-via-web-sys/42370/2
    // Generate cache of rays texture
    // generate rays
    // use ray_cache to calculate final ray hit
    // map polar coordinates to colors
}
