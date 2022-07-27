#version 300 es

precision mediump float;
uniform float r[K];
uniform float g[K];
uniform float b[K];
uniform sampler2D rtt_sampler;
out vec4 outColor;

vec4 weights(int i){
    return vec4(r[i],g[i],b[i],1.);
}

void main(){
    outColor=weights(0)*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,0))/1024.);
    for(int i=1;i<K;i++){
        vec4 weight=weights(i);
        #ifdef VERTICAL
        outColor+=weight*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,-i))/1024.);
        outColor+=weight*texture(rtt_sampler,(gl_FragCoord.xy+vec2(0,i))/1024.);
        #endif
        #ifdef HORIZONTAL
        outColor+=weight*texture(rtt_sampler,(gl_FragCoord.xy+vec2(-i,0))/1024.);
        outColor+=weight*texture(rtt_sampler,(gl_FragCoord.xy+vec2(i,0))/1024.);
        #endif
    }
}