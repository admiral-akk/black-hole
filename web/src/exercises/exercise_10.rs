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

fn complete(gl: &RenderContext, params: &BlackHoleParams) -> Vec<Data> {
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
    let fb3 = gl.create_framebuffer();

    let frag = SourceContext::new(include_str!("shaders/fragment/black_hole/complete.glsl"));
    gl.draw(None, &frag, &text, Some(&fb3.frame_buffer));

    let frame_buf_data = gl.read_from_frame_buffer(&fb3, 512, 512);

    let mut final_dirs = Vec::new();
    for i in 0..(frame_buf_data.len() / 4) {
        let s = &frame_buf_data[(4 * i)..(4 * i + 4)];
        if s[3] < 0.5 {
            continue;
        }
        let v = Vec3::new(s[0], s[1], s[2]);
        final_dirs.push(Data::Polar(i, v.to_polar()));
    }
    console_log!("New code!");
    final_dirs
}

pub fn exercise_10(gl: &RenderContext, params: &BlackHoleParams) {
    let mut data = complete(gl, params);

    // get the polar_coordinates -> colors
    let uv = generate_uv(params.dimensions.x as u32, params.dimensions.y as u32);
    let mut stars = Stars::new_from_u8(uv, params.dimensions.x as u32, params.dimensions.y as u32);
    stars.update_position(&&params.normalized_pos);
    stars.to_rgba(&mut data);

    // apply the colors to image

    let mut image_data = ImageData::new(params.dimensions.x as usize, params.dimensions.y as usize);
    image_data.load_colors(&data);
    let image = generate_texture_from_u8(&gl.gl, image_data.get_image(), 512);
    let image_context = UniformContext::new_from_allocated_ref(&image, "rtt_sampler");

    // need:

    //

    // Try this to load imagE: https://stackoverflow.com/questions/70309403/updating-html-canvas-imagedata-using-rust-webassembly
    // or this: https://users.rust-lang.org/t/a-future-for-loading-images-via-web-sys/42370/2
    // Generate cache of rays texture
    // generate rays
    // use ray_cache to calculate final ray hit
    // map polar coordinates to colors
    let frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
    gl.draw(None, &frag, &[&image_context], None);
    gl.delete_texture(&image);
}
