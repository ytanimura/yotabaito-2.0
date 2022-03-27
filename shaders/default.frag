#version 300 es
precision highp float;

uniform vec3 iResolution;
uniform float iTime;

out vec4 outColor;

void mainImage(out vec4 fragColor, in vec2 fragCoord);
void main() { mainImage(outColor, gl_FragCoord.xy); }

void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
    vec2 uv = fragCoord/iResolution.xy;
    vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));
    fragColor = vec4(col,1.0);
}