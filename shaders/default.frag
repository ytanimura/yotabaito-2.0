#version 300 es
precision highp float;

uniform vec3 iResolution;
uniform float iTime;

out vec4 outColor;

void mainImage(out vec4 fragColor, in vec2 fragCoord);
void main() { mainImage(outColor, gl_FragCoord.xy); }

void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
    fragColor = vec4(0.9, 0.9, 0.9, 1.0);
}
