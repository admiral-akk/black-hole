// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct RenderParams {
    cursor_pos: vec2<f32>,
    @align(8) resolution: vec2<f32>,
}

struct BlackHole {
    radius: f32,
    @align(4) temp:f32,
    @align(8) cursor_pos: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}
@group(0) @binding(0)
var t_diffuse: texture_1d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var<uniform> black_hole: BlackHole;
@group(0) @binding(3)
var<uniform> render_params: RenderParams;

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4((textureSample(t_diffuse, s_diffuse, in.tex_coords.x).r + 
    black_hole.radius) / 2.,render_params.cursor_pos.y / render_params.resolution.y,render_params.cursor_pos.x / render_params.resolution.x,1.);
}
 