#version 300 es

precision mediump float;
uniform sampler2D rtt_sampler;
out vec4 outColor;
void main(){
    outColor=texture(rtt_sampler,gl_FragCoord.xy/1024.);
}