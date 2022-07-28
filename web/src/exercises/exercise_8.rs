use rendering::structs::{
    data::Data, image_data::ImageData, observer::Observer, ray_cache::RayCache, stars::Stars,
};

use crate::{
    framework::{
        render_context::RenderContext, source_context::SourceContext,
        texture_utils::generate_texture_from_u8, uniform_context::UniformContext,
    },
    BlackHoleParams,
};

const RENDER_TEXTURE_DEFAULT: &str = include_str!("shaders/fragment/render_texture.glsl");

pub fn exercise_8(
    gl: &RenderContext,
    image_data: &mut ImageData,
    stars: &Stars,
    ray_cache: &RayCache,
    observer: &Observer,
    _params: &BlackHoleParams,
) {
    let mut data = vec![Data::None; image_data.get_sample_count()];
    // get the view_port -> start_dir
    observer.to_start_dir(&image_data.samples, &mut data);

    // get the start_dir -> final_dir
    // get the final_dir -> polar coordinates
    ray_cache.calculate_final_dir(&mut data);

    // get the polar_coordinates -> colors
    stars.to_rgba(&mut data);

    // apply the colors to image
    image_data.load_colors(&data);
    let image = generate_texture_from_u8(&gl.gl, image_data.get_image(), 1024);
    let image_context = UniformContext::new_from_allocated_ref(&image, "rtt_sampler", 1024, 1024);

    let frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
    gl.draw(None, &frag, &[&image_context], None);
    gl.delete_texture(&image);
}
