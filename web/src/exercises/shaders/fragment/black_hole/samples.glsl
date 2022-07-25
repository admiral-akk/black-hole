#version 300 es

precision mediump float;
out vec4 outColor;

uniform ivec2 dimensions;
uniform float distance;
uniform float vertical_fov_degrees;
uniform float black_hole_radius;
uniform int cache_width;
uniform vec3 normalized_pos;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;

void main(){
    vec2 delta=vec2(1./float(dimensions.x),1./float(dimensions.y));
    vec2 coord=(gl_FragCoord.xy)*delta;
    outColor=vec4(coord.x,coord.y,0.,1.);
}