#version 300 es

precision mediump float;
out vec4 outColor;
uniform float color_seed[3];
uniform float pos_seed[2];

float rand(vec2 xy,float seed){
    vec2 pos_s=vec2(pos_seed[0],pos_seed[1]);
    return fract(4123.12*sin(seed*dot(xy,pos_s)));
}

vec3 rand_xy(vec2 xy){
    return vec3(rand(xy,color_seed[0]),rand(xy,color_seed[1]),rand(xy,color_seed[2]));
}

void main(){
    vec3 r=rand_xy(gl_FragCoord.xy/512.);
    outColor=vec4(r.xyz,1.);
}