// Mountain range "Senkai"
// Copyright © 2022 Dr. Yoshinori Tanimura
// Attribution 4.0 International (CC BY 4.0)

// Hash without Sine https://www.shadertoy.com/view/4djSRW
float hash(vec2 p) {
    vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

float noise(vec2 p) {
    float it = floor(p.x), ft = fract(p.x);
    return (
          hash(vec2(it - 2.0, p.y)) * (1.0 - ft) * (1.0 - ft)
        + hash(vec2(it - 1.0, p.y)) * ((1.0 + ft) * (1.0 - ft) + ft * (2.0 - ft))
        + hash(vec2(it      , p.y)) * ft * ft
    ) / 2.0;
}

float fbm(vec2 p) {
    float s = 0.0;
    for (int i = 0; i < 5; i++) {
        s += noise(p);
        p.x *= 2.0;
    }
    return s;
}

void mainImage(out vec4 O, in vec2 U) {
    U /= iResolution.y;
    float time = mod(iTime, 200.0);
    vec3 icol = vec3(0);
    for (float i = 1.0; i <= 10.0; i += 1.0) {
        float height = fbm(vec2(U.x + time / (4.0 * i), i)) / 5.0;
        height = (0.8 + 0.4 * (i - 1.0)) * height - 0.3;
        float c = smoothstep(height + 0.005, height - 0.005, U.y);
        icol += max(vec3(c) * pow(2.0, - i + 1.0), icol) - icol;
    }

    O = vec4(1.0 - icol, 1);
}
