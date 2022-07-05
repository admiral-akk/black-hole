#version 300 es

precision mediump float;
uniform float w[3];
uniform sampler2D rtt_sampler;
out vec4 outColor;
void main(){
    outColor=w[0]*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,0))/512.);
    for(int i=1;i<=2;i++){
        outColor+=w[i]*texture(rtt_sampler,(gl_FragCoord.xy+vec2(-i,0))/512.);
        outColor+=w[i]*texture(rtt_sampler,(gl_FragCoord.xy+vec2(i,0))/512.);
    }
}