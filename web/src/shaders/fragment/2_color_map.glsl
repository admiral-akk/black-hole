#version 300 es

precision mediump float;
out vec4 outColor;

uniform sampler2D u_palette_1;
uniform sampler2D u_palette_2;

void main(){
    float index=mod(gl_FragCoord.x+gl_FragCoord.y,256.);
    // Cannot conditionally sample textures
    // Have to sample both of them?
    vec2 p=vec2((index+.5)/256.,.5);
    vec4 c1=texture(u_palette_1,p);
    vec4 c2=texture(u_palette_2,p);
    outColor=gl_FragCoord.x<256.?c1:c2;
}