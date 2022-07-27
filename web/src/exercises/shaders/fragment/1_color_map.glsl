#version 300 es

precision mediump float;
out vec4 outColor;

uniform sampler2D u_palette;

void main(){
    float index=(cos((gl_FragCoord.x+gl_FragCoord.y)*.1227184630308513)+1.)*128.;
    outColor=texture(u_palette,vec2((index+.5)/512.,.5));
}