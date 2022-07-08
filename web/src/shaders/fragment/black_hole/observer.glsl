#version 300 es

precision mediump float;
out vec4 outColor;

uniform sampler2D u_palette;

uniform ivec2 dimensions;
uniform float distance;
uniform float vertical_fov_degrees;
uniform float black_hole_radius;
uniform int cache_width;
uniform vec3 normalized_pos;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;

void main(){
    int x=dimensions.x;
    float d=distance+float(x)+float(dimensions.y);
    float f=d+vertical_fov_degrees;
    float b=f+black_hole_radius;
    float c=d+float(cache_width);
    float p=c+normalized_pos.x;
    float n=p+normalized_dir.x;
    float u=n+normalized_up.x;
    outColor=vec4(1.,u,0.,1.);
}