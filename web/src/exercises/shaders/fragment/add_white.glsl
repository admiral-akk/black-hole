#version 300 es

precision mediump float;
uniform sampler2D rtt_sampler;
out vec4 outColor;
void main(){
    outColor.rgb=texture(rtt_sampler,(gl_FragCoord.xy)/1024.).rgb+vec3(.01,.01,.01);
    outColor.a=1.;
}