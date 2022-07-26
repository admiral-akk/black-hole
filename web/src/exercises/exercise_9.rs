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

fn gpu_samples(
    gl: &RenderContext,
    params: &BlackHoleParams,
    text: &Vec<&UniformContext>,
    fb: &FrameBufferContext,
    image_data: &ImageData,
) -> Vec<Data> {
    let frag = SourceContext::new(include_str!("shaders/fragment/black_hole/samples.glsl"));
    gl.draw(None, &frag, &text, Some(&fb.frame_buffer));
    let frame_buf_data = gl.read_from_frame_buffer(&fb, 512, 512);

    let mut samples = Vec::new();
    for i in 0..(frame_buf_data.len() / 4) {
        let s = &frame_buf_data[(4 * i)..(4 * i + 4)];
        samples.push(Data::Sample(i, s[0], s[1]));
    }

    for i in 0..samples.len() {
        let expected = &image_data.samples[i];
        match expected {
            Data::Sample(i1, x1, y1) => {
                let actual = &samples[i];
                match actual {
                    Data::Sample(i2, x2, y2) => {
                        if i1 != i2 {
                            console_log!(
                                "\nIndicies differ! Expected: {:?}\nActual: {:?}\n",
                                expected,
                                actual
                            );
                            panic!();
                        }
                        if (x1 - x2).abs() > 0.0001 {
                            console_log!(
                                "\nX values differ! Expected: {:?}\nActual: {:?}\n",
                                expected,
                                actual
                            );
                            panic!();
                        }
                        if (y1 - y2).abs() > 0.0001 {
                            console_log!(
                                "\nY values differ! Expected: {:?}\nActual: {:?}\n",
                                expected,
                                actual
                            );
                            panic!();
                        }
                    }
                    _ => {
                        console_log!("\nNon sample in webgl samples!");
                        panic!("Non sample in webgl samples!")
                    }
                }
            }
            _ => {
                console_log!("\nNon sample in webgl samples!");
                panic!("Non sample in image samples!")
            }
        }
    }
    samples
}

fn observer(
    gl: &RenderContext,
    params: &BlackHoleParams,
    text: &Vec<&UniformContext>,
    samples: &Vec<Data>,
    fb: &FrameBufferContext,
    image_data: &ImageData,
) -> Vec<Data> {
    let frag = SourceContext::new(include_str!("shaders/fragment/black_hole/observer.glsl"));

    let mut data = vec![Data::None; image_data.get_sample_count()];
    gl.draw(None, &frag, &text, Some(&fb.frame_buffer));
    let frame_buf_data = gl.read_from_frame_buffer(&fb, 512, 512);
    // get the view_port -> start_dir
    let observer = Observer::new(
        params.normalized_pos,
        params.normalized_dir,
        params.normalized_up,
        params.vertical_fov_degrees,
    );
    observer.to_start_dir(&samples, &mut data);
    let mut start_dirs = Vec::new();
    for i in 0..(frame_buf_data.len() / 4) {
        let s = &frame_buf_data[(4 * i)..(4 * i + 4)];
        start_dirs.push(Data::ObserverDir(i, Vec3::new(s[0], s[1], s[2])));
    }

    for i in 0..data.len() {
        let expected = &data[i];
        match expected {
            Data::ObserverDir(i1, v1) => {
                let actual = &start_dirs[i];
                match actual {
                    Data::ObserverDir(i2, v2) => {
                        if i1 != i2 {
                            console_log!(
                                "\nIndicies differ! Expected: {:?}\nActual: {:?}\n",
                                expected,
                                actual
                            );
                            panic!();
                        }
                        if (*v1 - *v2).length() > 0.0001 {
                            console_log!(
                                "\nVectors values differ! Expected: {:?}\nActual: {:?}\n",
                                expected,
                                actual
                            );
                            panic!();
                        }
                    }
                    _ => {
                        console_log!("\nNon ObserverDir in webgl samples!");
                        panic!("Non ObserverDir in webgl samples!")
                    }
                }
            }
            _ => {
                console_log!("\nNon ObserverDir in image samples!");
                panic!("Non ObserverDir in image samples!")
            }
        }
    }

    start_dirs
}

fn final_dir(
    gl: &RenderContext,
    params: &BlackHoleParams,
    text: &Vec<&UniformContext>,
    samples: &Vec<Data>,
    fb: &FrameBufferContext,
    fb2: &FrameBufferContext,
    fb3: &FrameBufferContext,
    ray_cache: &RayCache,
    image_data: &ImageData,
    data: &mut Vec<Data>,
) -> Vec<Data> {
    let final_dirs: Vec<Vec3> = ray_cache.cache.iter().map(|r| r.final_dir).collect();
    let mut f32_vec: Vec<f32> = Vec::new();
    for i in 0..final_dirs.len() {
        let final_dir = final_dirs[i];
        f32_vec.push(final_dir.x);
        f32_vec.push(final_dir.y);
        f32_vec.push(final_dir.z);
        f32_vec.push(1.0);
    }

    let fb3 = gl.create_framebuffer();

    let frag = SourceContext::new(include_str!("shaders/fragment/black_hole/final_dir.glsl"));
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
    // get the start_dir -> final_dir
    // get the final_dir -> polar coordinates
    ray_cache.calculate_final_dir(data);

    if data.len() != final_dirs.len() {
        console_log!(
            "\nLengths differ! Expected: {}\nActual: {}\n",
            data.len(),
            final_dirs.len()
        );
        panic!("Non Polar in image samples!")
    }

    for i in 0..final_dirs.len() {
        let expected = &data[i];
        match expected {
            Data::Polar(i1, polar1) => {
                let actual = &final_dirs[i];
                match actual {
                    Data::Polar(i2, polar2) => {
                        if i1 != i2 {
                            console_log!(
                                "\nIndicies differ! \nExpected: {:?}\nActual: {:?}\n",
                                expected,
                                actual
                            );
                            panic!();
                        }
                        if (polar1.phi - polar2.phi).abs() > 0.05 {
                            console_log!(
                                "\n phi values differ! \nExpected: {:?}\nActual: {:?}\n ",
                                expected,
                                actual
                            );
                            panic!();
                        }
                        if (polar1.theta - polar2.theta).abs() > 0.05 {
                            console_log!(
                                "\n theta values differ! \nExpected: {:?}\nActual: {:?}\n",
                                expected,
                                actual
                            );
                            panic!();
                        }
                    }
                    _ => {
                        console_log!("\nNon Polar in webgl samples!");
                        panic!("Non Polar in webgl samples!")
                    }
                }
            }
            _ => {
                console_log!("\nNon Polar in image samples!");
                panic!("Non Polar in image samples!")
            }
        }
    }
    final_dirs
}

fn step_by_step(gl: &RenderContext, params: &BlackHoleParams) {
    let mut image_data = ImageData::new(params.dimensions.x as usize, params.dimensions.y as usize);
    let uniforms = params.uniform_context();
    let mut text: Vec<&UniformContext> = uniforms.iter().map(|u| u).collect();
    let fb = gl.create_framebuffer();
    let fb_context =
        UniformContext::new_from_allocated_ref(&fb.backing_texture, "requested_samples");
    let samples = gpu_samples(gl, params, &text, &fb, &image_data);
    text.push(&fb_context);

    let fb2 = gl.create_framebuffer();
    let mut data = observer(gl, params, &text, &samples, &fb2, &image_data);
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

    data = final_dir(
        gl,
        params,
        &mut text,
        &samples,
        &fb,
        &fb2,
        &fb3,
        &ray_cache,
        &image_data,
        &mut data,
    );

    // get the polar_coordinates -> colors
    let uv = generate_uv(params.dimensions.x as u32, params.dimensions.y as u32);
    let mut stars = Stars::new_from_u8(uv, params.dimensions.x as u32, params.dimensions.y as u32);

    stars.update_position(&&params.normalized_pos);
    stars.to_rgba(&mut data);

    // apply the colors to image
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
    text.push(&image_context);
    let frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
    gl.draw(None, &frag, &text, None);
    gl.delete_texture(&image);
}

pub fn exercise_9(gl: &RenderContext, params: &BlackHoleParams) {
    step_by_step(gl, params);
}
