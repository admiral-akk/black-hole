#version 300 es

precision mediump float;
uniform sampler2D rtt_sampler;
out vec4 outColor;
void main(){
    outColor=(
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,0))/1024.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,1))/1024.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-1))/1024.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,2))/1024.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-2))/1024.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,3))/1024.)+
        texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-3))/1024.)
    )/7.;
}