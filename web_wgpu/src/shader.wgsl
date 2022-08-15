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
    observer_matrix: mat4x4<f32>,
    cursor_pos: vec2<f32>,
    resolution: vec2<f32>,
    distance: f32,
    time_s: f32,
}

struct BlackHole {
    disc_bounds: vec2<f32>,
    distance_bounds: vec2<f32>,
    radius: f32,
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
var galaxy_t: texture_2d<f32>;
@group(0) @binding(1)
var galaxy_s: sampler;
@group(0) @binding(2)
var<uniform> black_hole: BlackHole;
@group(0) @binding(3)
var<uniform> render_params: RenderParams;
@group(0) @binding(4)
var noise_t: texture_2d<f32>;
@group(0) @binding(5)
var noise_s: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let min_dim=min(render_params.resolution.x,render_params.resolution.y);
        let max_dim=max(render_params.resolution.x,render_params.resolution.y);
        let diff=max_dim-min_dim;
        let lower=diff/2.;
        let upper=min_dim+lower;
        var offset=vec2((max_dim-min_dim)/2.);
        let delta=1./vec2(min_dim);
        if(render_params.resolution.x>render_params.resolution.y){
            if(in.tex_coords.x *render_params.resolution.x <lower||in.tex_coords.x*render_params.resolution.x>upper){
                return vec4(vec3(0.),1.);
            } else {
            offset.y=0.;

            }
        }else{
            if(in.tex_coords.y*render_params.resolution.y<lower||in.tex_coords.y*render_params.resolution.y>upper){
                return vec4(vec3(0.),1.);
            } else {
                
            offset.x=0.;
            }
        }
        
    let v = (render_params.observer_matrix*vec4(0.,0.,-1.,0.)).xyz;
    return vec4(v.x,v.y,v.z,1.);
}
 