#version 300 es

precision mediump float;
uniform float w[K];
uniform sampler2D rtt_sampler;
out vec4 outColor;
void main(){
    outColor=w[0]*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,0))/1024.);
    for(int i=1;i<K;i++){
        #ifdef VERTICAL
        outColor+=w[i]*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-i))/1024.);
        outColor+=w[i]*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,i))/1024.);
        #endif
        #ifdef HORIZONTAL
        outColor+=w[i]*texture(rtt_sampler,(gl_FragCoord.xy+vec2(-i,0))/1024.);
        outColor+=w[i]*texture(rtt_sampler,(gl_FragCoord.xy+vec2(i,0))/1024.);
        #endif
    }
}