#version 300 es

precision mediump float;
uniform float v[3];
out vec4 outColor;
void main(){
    outColor=vec4(v[0],v[1],v[2],1.);
}