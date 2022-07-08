#version 300 es

precision mediump float;
uniform sampler2D requested_samples;
out vec4 outColor;

uniform ivec2 dimensions;
uniform float distance;
uniform float vertical_fov_degrees;
uniform float black_hole_radius;
uniform int cache_width;
uniform vec3 normalized_pos;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform mat3x3 observer_mat;
#define PI 3.1415926538
void main(){
    vec2 delta=vec2(1./float(dimensions.x),1./float(dimensions.y));
    vec2 sample_coord=texture(requested_samples,gl_FragCoord.xy*delta).xy;
    
    vec3 forward=observer_mat*normalized_dir;
    vec3 up=observer_mat*normalized_up;
    vec3 right=cross(forward,up);
    
    float view_width=2.*tan(PI*vertical_fov_degrees/360.);
    
    vec3 v=view_width*((sample_coord.x-.5)*right+(sample_coord.y-.5)*up)+forward;
    outColor=vec4(v.xyz,1.);
}