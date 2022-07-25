#version 300 es

precision mediump float;
uniform sampler2D rtt_sampler;
out vec4 outColor;
void main(){
    outColor=(
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,0))/512.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,1))/512.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-1))/512.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,2))/512.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-2))/512.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,3))/512.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-3))/512.)
    )/7.;
}