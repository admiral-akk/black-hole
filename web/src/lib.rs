extern crate cfg_if;
extern crate wasm_bindgen;

mod framework;
mod utils;

use framework::frame_buffer_context::FrameBufferContext;
use framework::texture_utils::generate_texture_from_f32;
use glam::IVec2;
use glam::Mat3;
use glam::Quat;
use glam::Vec2;
use glam::Vec3;
use rendering::render::render;
use rendering::structs::data::Data;
use rendering::structs::image_data::ImageData;
use rendering::structs::observer;
use rendering::structs::observer::Observer;
use rendering::structs::ray_cache::RayCache;
use rendering::structs::stars::Stars;
use rendering::utils::extensions::ToPolar;
use wasm_timer::SystemTime;
use web_sys::console;
use web_sys::WebGlRenderingContext;
use web_sys::WebGlTexture;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use cfg_if::cfg_if;
use framework::render_context::RenderContext;
use framework::source_context::SourceContext;
use framework::uniform_context::UniformContext;
use utils::color_map::colormap1;
use utils::color_map::colormap2;
use utils::gaussian::generate_gaussian_weights;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlOptionElement;

use crate::framework::texture_utils::generate_texture_from_u8;
use image::io::Reader;

// https://rustwasm.github.io/wasm-bindgen/exbuild/webgl/
// https://webglfundamentals.org/webgl/lessons/webgl-fundamentals.html
// https://michaelerule.github.io/webgpgpu/examples/Example_1_hello_gpu.html
// https://github.com/michaelerule/webgpgpu/blob/master/examples/Example_1_hello_gpu.html
// https://github.com/michaelerule/webgpgpu
cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn cancelInterval(token: f64);
    fn requestAnimationFrame(closure: &Closure<dyn FnMut()>) -> u32;
    fn cancelAnimationFrame(id: u32);
}

fn get_selected_index() -> Result<u32, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    Ok(document
        .get_element_by_id("input")
        .unwrap()
        .dyn_into::<web_sys::HtmlSelectElement>()?
        .selected_index() as u32)
}

enum ExerciseState {
    Exercise0,
    Exercise1(WebGlTexture),
    Exercise2(WebGlTexture, WebGlTexture),
    Exercise3(FrameBufferContext),
    Exercise4(FrameBufferContext, FrameBufferContext, Vec<f32>),
    Exercise5(
        FrameBufferContext,
        FrameBufferContext,
        Vec<f32>,
        Vec<f32>,
        Vec<f32>,
    ),
    Exercise6,
    Exercise7(FrameBufferContext, FrameBufferContext),
    Exercise8(ImageData, Stars, RayCache, Observer, BlackHoleParams),
    Exercise9(BlackHoleParams),
}

impl Default for ExerciseState {
    fn default() -> Self {
        ExerciseState::Exercise0
    }
}

struct BlackHoleParams {
    pub dimensions: IVec2,
    pub distance: f32,
    pub vertical_fov_degrees: f32,
    pub black_hole_radius: f32,
    pub cache_width: i32,
    pub normalized_pos: Vec3,
    pub normalized_dir: Vec3,
    pub normalized_up: Vec3,
    pub observer_mat: Mat3,
}

impl BlackHoleParams {
    fn new(
        dimensions: IVec2,
        distance: f32,
        vertical_fov_degrees: f32,
        black_hole_radius: f32,
        cache_width: i32,
        pos: Vec3,
        dir: Vec3,
        up: Vec3,
    ) -> Self {
        let observer_quat = Quat::from_rotation_arc(pos.normalize(), -Vec3::Z);
        let euler = Quat::to_euler(observer_quat, glam::EulerRot::XYZ);
        let observer_mat = Mat3::from_euler(glam::EulerRot::XYZ, euler.0, euler.1, euler.2);

        Self {
            dimensions,
            distance,
            vertical_fov_degrees,
            black_hole_radius,
            cache_width,
            normalized_pos: pos.normalize(),
            normalized_dir: dir.normalize(),
            normalized_up: up.normalize(),
            observer_mat,
        }
    }

    fn uniform_context(&self) -> Vec<UniformContext> {
        let mut v = Vec::new();
        v.push(UniformContext::ivec2(self.dimensions, "dimensions"));
        v.push(UniformContext::f32(self.distance, "distance"));
        v.push(UniformContext::f32(
            self.vertical_fov_degrees,
            "vertical_fov_degrees",
        ));
        v.push(UniformContext::f32(
            self.black_hole_radius,
            "black_hole_radius",
        ));
        v.push(UniformContext::i32(self.cache_width, "cache_width"));
        v.push(UniformContext::vec3(self.normalized_pos, "normalized_pos"));
        v.push(UniformContext::vec3(self.normalized_dir, "normalized_dir"));
        v.push(UniformContext::vec3(self.normalized_up, "normalized_up"));
        v.push(UniformContext::mat3x3(self.observer_mat, "observer_mat"));
        v
    }
}

impl ExerciseState {
    pub fn index(&self) -> u32 {
        match self {
            ExerciseState::Exercise0 => 0,
            ExerciseState::Exercise1(..) => 1,
            ExerciseState::Exercise2(..) => 2,
            ExerciseState::Exercise3(..) => 3,
            ExerciseState::Exercise4(..) => 4,
            ExerciseState::Exercise5(..) => 5,
            ExerciseState::Exercise6 => 6,
            ExerciseState::Exercise7(..) => 7,
            ExerciseState::Exercise8(..) => 8,
            ExerciseState::Exercise9(..) => 9,
        }
    }
}

fn init_exercise(gl: &RenderContext, exercise_state: &mut ExerciseState, exercise_index: u32) {
    match exercise_index {
        0 => {
            *exercise_state = ExerciseState::Exercise0;
        }
        1 => {
            let cm = generate_texture_from_u8(&gl.gl, &colormap1(), 256);
            *exercise_state = ExerciseState::Exercise1(cm);
        }
        2 => {
            let cm1 = generate_texture_from_u8(&gl.gl, &colormap1(), 256);
            let cm2 = generate_texture_from_u8(&gl.gl, &colormap2(), 256);
            *exercise_state = ExerciseState::Exercise2(cm1, cm2);
        }
        3 => {
            *exercise_state = ExerciseState::Exercise3(gl.create_framebuffer());
        }
        4 => {
            *exercise_state = ExerciseState::Exercise4(
                gl.create_framebuffer(),
                gl.create_framebuffer(),
                generate_gaussian_weights(1.0, 3),
            );
        }
        5 => {
            *exercise_state = ExerciseState::Exercise5(
                gl.create_framebuffer(),
                gl.create_framebuffer(),
                generate_gaussian_weights(1.0, 3),
                generate_gaussian_weights(2.0, 3),
                generate_gaussian_weights(3.0, 3),
            );
        }
        6 => {
            *exercise_state = ExerciseState::Exercise6;
        }
        7 => {
            *exercise_state =
                ExerciseState::Exercise7(gl.create_framebuffer(), gl.create_framebuffer());
        }
        8 => {
            let distance = 3.0;
            let vertical_fov_degrees = 120.0;
            let black_hole_radius = 1.5;
            let cache_width: i32 = 1024;
            let (pos, dir, up) = (distance * Vec3::Z, -Vec3::Z, Vec3::Y);
            let params = BlackHoleParams::new(
                IVec2::new(512, 512),
                distance,
                vertical_fov_degrees,
                black_hole_radius,
                cache_width,
                pos,
                dir,
                up,
            );
            let uv = generate_uv(params.dimensions.x as u32, params.dimensions.y as u32);

            let mut stars =
                Stars::new_from_u8(uv, params.dimensions.x as u32, params.dimensions.y as u32);
            let ray_cache = RayCache::compute_new(
                params.cache_width as usize,
                params.black_hole_radius,
                params.distance,
            );

            let observer = Observer::new(
                params.normalized_pos,
                params.normalized_dir,
                params.normalized_up,
                params.vertical_fov_degrees,
            );
            stars.update_position(&&params.normalized_pos);
            let image_data =
                ImageData::new(params.dimensions.x as usize, params.dimensions.y as usize);
            *exercise_state =
                ExerciseState::Exercise8(image_data, stars, ray_cache, observer, params);
        }
        9 => {
            let distance = 3.0;
            let vertical_fov_degrees = 120.0;
            let black_hole_radius = 1.5;
            let cache_width: i32 = 1024;
            let (pos, dir, up) = (distance * Vec3::Z, -Vec3::Z, Vec3::Y);
            let params = BlackHoleParams::new(
                IVec2::new(512, 512),
                distance,
                vertical_fov_degrees,
                black_hole_radius,
                cache_width,
                pos,
                dir,
                up,
            );
            *exercise_state = ExerciseState::Exercise9(params);
        }
        _ => {}
    }
}

pub struct RenderState {
    gl: RenderContext,
    prev_params: Cell<RenderParams>,
    exercise_state: RefCell<Box<ExerciseState>>,
}

fn clean_up_exercise(gl: &RenderContext, exercise_state: &mut ExerciseState) {
    match exercise_state {
        ExerciseState::Exercise0 => {}
        ExerciseState::Exercise1(cm) => {
            gl.delete_texture(&cm);
        }
        ExerciseState::Exercise2(cm1, cm2) => {
            gl.delete_texture(&cm1);
            gl.delete_texture(&cm2);
        }
        ExerciseState::Exercise3(fb) => {
            gl.delete_framebuffer(&fb);
        }
        ExerciseState::Exercise4(fb1, fb2, _) => {
            gl.delete_framebuffer(&fb1);
            gl.delete_framebuffer(&fb2);
        }
        ExerciseState::Exercise5(fb1, fb2, _, _, _) => {
            gl.delete_framebuffer(&fb1);
            gl.delete_framebuffer(&fb2);
        }
        ExerciseState::Exercise6 => {}
        ExerciseState::Exercise7(fb1, fb2) => {
            gl.delete_framebuffer(&fb1);
            gl.delete_framebuffer(&fb2);
        }
        ExerciseState::Exercise8(..) => {}
        ExerciseState::Exercise9(..) => {}
        _ => {}
    }
}

fn update_exercise(
    gl: &RenderContext,
    exercise_state: &mut ExerciseState,
    new_params: &RenderParams,
) {
    if exercise_state.index() != new_params.select_index {
        clean_up_exercise(gl, exercise_state);
        init_exercise(gl, exercise_state, new_params.select_index);
    }
}

fn render_exercise(gl: &RenderContext, exercise_state: &mut ExerciseState) {
    let mut frag;
    match exercise_state {
        ExerciseState::Exercise0 => {
            frag = SourceContext::new(include_str!("shaders/fragment/striped.glsl"));
            gl.draw(None, &frag, &[], None);
        }
        ExerciseState::Exercise1(cm) => {
            frag = SourceContext::new(include_str!("shaders/fragment/1_color_map.glsl"));
            let cm_context = UniformContext::new_from_allocated_ref(&cm, "u_palette");
            gl.draw(None, &frag, &[&cm_context], None);
        }
        ExerciseState::Exercise2(cm1, cm2) => {
            frag = SourceContext::new(include_str!("shaders/fragment/2_color_map.glsl"));
            let cm_context1 = UniformContext::new_from_allocated_ref(&cm1, "u_palette_1");
            let cm_context2 = UniformContext::new_from_allocated_ref(&cm2, "u_palette_2");
            gl.draw(None, &frag, &[&cm_context1, &cm_context2], None);
        }
        ExerciseState::Exercise3(fb) => {
            frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
            let fb_texture =
                UniformContext::new_from_allocated_ref(&fb.backing_texture, "rtt_sampler");
            gl.draw(None, &frag, &[], Some(&fb.frame_buffer));

            frag = SourceContext::new(include_str!("shaders/fragment/blur.glsl"));
            gl.draw(None, &frag, &[&fb_texture], None);
        }
        ExerciseState::Exercise4(fb1, fb2, kernel) => {
            let fb_texture =
                UniformContext::new_from_allocated_ref(&fb1.backing_texture, "rtt_sampler");
            let fb_texture2 =
                UniformContext::new_from_allocated_ref(&fb2.backing_texture, "rtt_sampler");
            let kernel_weights = UniformContext::array_f32(&kernel, "w");

            frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
            gl.draw(None, &frag, &[], Some(&fb1.frame_buffer));

            for _ in 0..10 {
                frag = SourceContext::new(include_str!("shaders/fragment/gaussian_blur.glsl"));
                frag.add_parameter("HORIZONTAL", "TRUE");
                frag.add_parameter("K", &kernel.len().to_string());
                gl.draw(
                    None,
                    &frag,
                    &[&fb_texture, &kernel_weights],
                    Some(&fb2.frame_buffer),
                );

                frag = SourceContext::new(include_str!("shaders/fragment/gaussian_blur.glsl"));
                frag.add_parameter("K", &kernel.len().to_string());
                frag.add_parameter("VERTICAL", "TRUE");
                gl.draw(
                    None,
                    &frag,
                    &[&fb_texture2, &kernel_weights],
                    Some(&fb1.frame_buffer),
                );
            }
            frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
            gl.draw(None, &frag, &[&fb_texture], None);
        }
        ExerciseState::Exercise5(fb1, fb2, r, g, b) => {
            let fb_texture =
                UniformContext::new_from_allocated_ref(&fb1.backing_texture, "rtt_sampler");
            let fb_texture2 =
                UniformContext::new_from_allocated_ref(&fb2.backing_texture, "rtt_sampler");
            let r_kernel_weights = UniformContext::array_f32(&r, "r");
            let g_kernel_weights = UniformContext::array_f32(&g, "g");
            let b_kernel_weights = UniformContext::array_f32(&b, "b");

            frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
            gl.draw(None, &frag, &[], Some(&fb1.frame_buffer));

            for _ in 0..10 {
                frag =
                    SourceContext::new(include_str!("shaders/fragment/multi_gaussian_blur.glsl"));
                frag.add_parameter("HORIZONTAL", "TRUE");
                frag.add_parameter("K", &r.len().to_string());
                gl.draw(
                    None,
                    &frag,
                    &[
                        &fb_texture,
                        &r_kernel_weights,
                        &g_kernel_weights,
                        &b_kernel_weights,
                    ],
                    Some(&fb2.frame_buffer),
                );

                frag =
                    SourceContext::new(include_str!("shaders/fragment/multi_gaussian_blur.glsl"));
                frag.add_parameter("K", &r.len().to_string());
                frag.add_parameter("VERTICAL", "TRUE");
                gl.draw(
                    None,
                    &frag,
                    &[
                        &fb_texture2,
                        &r_kernel_weights,
                        &g_kernel_weights,
                        &b_kernel_weights,
                    ],
                    Some(&fb1.frame_buffer),
                );
            }
            frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
            gl.draw(None, &frag, &[&fb_texture], None);
        }
        ExerciseState::Exercise6 => {
            let time = 1.0;

            let pos_seed = Vec2::new(52.912 * time, 11.30 * time);
            let color_seed = Vec3::new(10.5121 * time, 22.958 * time, 25.1 * time);

            frag = SourceContext::new(include_str!("shaders/fragment/psuedo_random.glsl"));
            let pos_seed_uniform = UniformContext::vec2(pos_seed, "pos_seed");
            let color_seed_uniform = UniformContext::vec3(color_seed, "color_seed");
            gl.draw(None, &frag, &[&pos_seed_uniform, &color_seed_uniform], None);
        }
        ExerciseState::Exercise7(fb1, fb2) => {
            let state_texture =
                UniformContext::new_from_allocated_ref(&fb1.backing_texture, "rtt_sampler");
            frag = SourceContext::new(include_str!("shaders/fragment/add_white.glsl"));
            gl.draw(None, &frag, &[&state_texture], Some(&fb2.frame_buffer));

            let fb_texture =
                UniformContext::new_from_allocated_ref(&fb2.backing_texture, "rtt_sampler");
            gl.draw(None, &frag, &[&fb_texture], Some(&fb1.frame_buffer));
            gl.draw(None, &frag, &[&fb_texture], None);
        }
        ExerciseState::Exercise8(image_data, stars, ray_cache, observer, params) => {
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
            let image = generate_texture_from_u8(&gl.gl, image_data.get_image(), 512);
            let image_context = UniformContext::new_from_allocated_ref(&image, "rtt_sampler");

            frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
            gl.draw(None, &frag, &[&image_context], None);
            gl.delete_texture(&image);
        }
        ExerciseState::Exercise9(params) => {
            let uv = generate_uv(params.dimensions.x as u32, params.dimensions.y as u32);

            let mut stars =
                Stars::new_from_u8(uv, params.dimensions.x as u32, params.dimensions.y as u32);
            let ray_cache = RayCache::compute_new(
                params.cache_width as usize,
                params.black_hole_radius,
                params.distance,
            );

            let observer = Observer::new(
                params.normalized_pos,
                params.normalized_dir,
                params.normalized_up,
                params.vertical_fov_degrees,
            );
            stars.update_position(&&params.normalized_pos);
            let mut image_data =
                ImageData::new(params.dimensions.x as usize, params.dimensions.y as usize);

            let uniforms = params.uniform_context();
            let mut text: Vec<&UniformContext> = uniforms.iter().map(|u| u).collect();
            let fb = gl.create_framebuffer();
            let fb_context =
                UniformContext::new_from_allocated_ref(&fb.backing_texture, "requested_samples");
            let fb2 = gl.create_framebuffer();
            frag = SourceContext::new(include_str!("shaders/fragment/black_hole/samples.glsl"));
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

            let mut data = vec![Data::None; image_data.get_sample_count()];

            frag = SourceContext::new(include_str!("shaders/fragment/black_hole/observer.glsl"));

            text.push(&fb_context);
            gl.draw(None, &frag, &text, Some(&fb2.frame_buffer));
            let frame_buf_data = gl.read_from_frame_buffer(&fb2, 512, 512);
            // get the view_port -> start_dir
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

            for i in 0..start_dirs.len() {
                data[i] = start_dirs[i].clone();
            }

            let final_dirs: Vec<Vec3> = ray_cache.cache.iter().map(|r| r.final_dir).collect();
            let mut f32_vec: Vec<f32> = Vec::new();
            for i in 0..final_dirs.len() {
                let final_dir = final_dirs[i];
                f32_vec.push(final_dir.x);
                f32_vec.push(final_dir.y);
                f32_vec.push(final_dir.z);
                f32_vec.push(1.0);
            }

            console_log!("Ray cache len: {}", final_dirs.len());
            let ray_cache_tex =
                generate_texture_from_f32(&gl.gl, &f32_vec, final_dirs.len() as i32);
            let ray_context =
                UniformContext::new_from_allocated_ref(&ray_cache_tex, "ray_cache_tex");
            let ray_length = UniformContext::f32(final_dirs.len() as f32, "ray_cache_length");
            let max_z = UniformContext::f32(ray_cache.max_z, "max_z");
            let fb_context2 =
                UniformContext::new_from_allocated_ref(&fb2.backing_texture, "start_ray_tex");
            text.push(&ray_context);
            text.push(&ray_length);

            console_log!("Max z: {}", ray_cache.max_z);

            text.push(&max_z);
            text.push(&fb_context2);
            let fb3 = gl.create_framebuffer();

            frag = SourceContext::new(include_str!("shaders/fragment/black_hole/final_dir.glsl"));
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
            ray_cache.calculate_final_dir(&mut data);

            if (data.len() != final_dirs.len()) {
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
                                        "\nIndicies differ! \nExpected: {:?}\nActual: {:?}\nStart dir {:?}\n",
                                        expected,
                                        actual, start_dirs[i]
                                    );
                                    panic!();
                                }
                                if (polar1.phi - polar2.phi).abs() > 0.05 {
                                    console_log!(
                                            "\n phi values differ! \nExpected: {:?}\nActual: {:?}\n Start dir {:?}\n",
                                            expected,
                                            actual, start_dirs[i]
                                        );
                                    panic!();
                                }
                                if (polar1.theta - polar2.theta).abs() > 0.05 {
                                    console_log!(
                                            "\n theta values differ! \nExpected: {:?}\nActual: {:?}\n Start dir {:?}\n",
                                            expected,
                                            actual, start_dirs[i]
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
            for i in 0..final_dirs.len() {
                data[i] = final_dirs[i].clone();
            }

            // get the polar_coordinates -> colors
            stars.to_rgba(&mut data);

            // apply the colors to image
            image_data.load_colors(&data);
            let image = generate_texture_from_u8(&gl.gl, image_data.get_image(), 512);
            let image_context = UniformContext::new_from_allocated_ref(&image, "rtt_sampler");

            // need:

            //

            // Generate cache of rays texture
            // generate rays
            // use ray_cache to calculate final ray hit
            // map polar coordinates to colors
            text.push(&image_context);
            frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
            gl.draw(None, &frag, &text, None);
            gl.delete_texture(&image);
        }
    }
}

const EXERCISE_COUNT: u32 = 10;
const RENDER_TEXTURE_DEFAULT: &str = include_str!("shaders/fragment/render_texture.glsl");
impl RenderState {
    fn render(&self, params: &RenderParams) -> Result<(), JsValue> {
        console_log!("params: {:?}", params);
        let gl = &self.gl;

        update_exercise(gl, &mut *self.exercise_state.borrow_mut(), params);
        render_exercise(gl, &mut *self.exercise_state.borrow_mut());
        self.prev_params.set(*params);
        Ok(())
    }
}

impl RenderState {
    pub fn new(width: u32, height: u32) -> Result<RenderState, JsValue> {
        let gl = RenderContext::new(width, height);

        Ok(RenderState {
            gl,
            prev_params: Cell::default(),
            exercise_state: RefCell::default(),
        })
    }
}

fn generate_uv(width: u32, height: u32) -> Vec<u8> {
    let mut uv = Vec::new();
    for x in 0..width {
        for y in 0..height {
            let r = 255 * x / width;
            let g = 255 * y / height;
            let b = 0;
            let a = 255;
            uv.push(r as u8);
            uv.push(g as u8);
            uv.push(b as u8);
            uv.push(a as u8);
        }
    }

    uv
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[derive(Clone, Copy, Debug, Default)]
struct RenderParams {
    pub seconds_since_start: f32,
    pub select_index: u32,
    pub mouse_pos: Option<(i32, i32)>,
}

impl RenderParams {
    pub fn update_time(&self, seconds_since_start: f32) -> RenderParams {
        let mut c = self.clone();
        c.seconds_since_start = seconds_since_start;
        c
    }

    pub fn update_exercise(&self, select_index: u32) -> RenderParams {
        let mut c = self.clone();
        c.seconds_since_start = 0.0;
        c.select_index = select_index;
        c
    }

    pub fn update_mouse_pos(&self, mouse_pos: Option<(i32, i32)>) -> RenderParams {
        let mut c = self.clone();
        c.mouse_pos = mouse_pos;
        c
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = document();
    let select = document
        .get_element_by_id("input")
        .unwrap()
        .dyn_into::<web_sys::HtmlSelectElement>()?;
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    for i in 1..=EXERCISE_COUNT {
        let option = HtmlOptionElement::new_with_text(&format!("Exercise {}", i))?;
        select.append_child(&option)?;
    }

    let start_time = Rc::new(Cell::new(SystemTime::now()));
    let params = Rc::new(Cell::new(RenderParams::default()));
    let renderer = Rc::new(RenderState::new(512, 512)?);
    {
        let start_time = start_time.clone();
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            start_time.set(SystemTime::now());
            let exercise = get_selected_index().unwrap();
            params.set(params.get().update_exercise(exercise));
        }) as Box<dyn FnMut(_)>);
        select.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            params.set(
                params
                    .get()
                    .update_mouse_pos(Some((event.offset_x(), event.offset_y()))),
            );
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            params.set(params.get().update_mouse_pos(None));
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let render_func = Rc::new(RefCell::new(None));
    let g = render_func.clone();
    {
        let renderer = renderer.clone();
        let start_time = start_time.clone();
        let params = params.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let seconds_since_start = SystemTime::now()
                .duration_since(start_time.get())
                .unwrap()
                .as_secs_f32();
            params.set(params.get().update_time(seconds_since_start));
            renderer.render(&params.get()).unwrap();
            requestAnimationFrame(render_func.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    Ok(())
}
