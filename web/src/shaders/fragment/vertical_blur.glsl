#version 300 es

precision mediump float;
uniform sampler2D rtt_sampler;
out vec4 outColor;
void main(){
    outColor=(
        (1./16.)*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-2))/512.)+
        (4./16.)*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-1))/512.)+
        (6./16.)*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,0))/512.)+
        (4./16.)*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,1))/512.)+
        (1./16.)*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,2))/512.)
    );
}