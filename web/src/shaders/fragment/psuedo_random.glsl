#version 300 es

precision mediump float;
out vec4 outColor;
uniform vec3 color_seed;
uniform vec2 pos_seed;

float rand(vec2 xy,float seed){
    return fract(4123.12*sin(seed*dot(xy,pos_seed)));
}

vec3 rand_xy(vec2 xy){
    return vec3(rand(xy,color_seed.x),rand(xy,color_seed.y),rand(xy,color_seed.z));
}

void main(){
    vec3 r=rand_xy(gl_FragCoord.xy/512.);
    outColor=vec4(r.xyz,1.);
}