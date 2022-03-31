#iChannel0 "../dist/codepage12.png"

// Random Primitives
// Copyright © 2022 Dr. Yoshinori Tanimura
// Attribution 4.0 International (CC BY 4.0)

const float PI = 3.141592653;

const float DOT_RESOLUTION = 48.0;

// Hash without Sine https://www.shadertoy.com/view/4djSRW
vec2 hash21(float p) {
    vec3 p3 = fract(vec3(p) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}

int random(in vec2 uv) {
    float time = iTime * 10.0;
    float x = fract(sin(dot(uv + floor(time      ) * 0.1, vec2(11.4, 51.4))) * 1919.810);
    float y = fract(sin(dot(uv + floor(time + 1.0) * 0.1, vec2(11.4, 51.4))) * 1919.810);
    return mix(x, y, fract(time)) < 0.5 ? 0 : 1;
}

int circleWave(in vec2 uv) {
    float mag = fract(length(uv) * 4.0 - iTime);
    return mag < 0.5 ? 1 : 0;
}

int lissajous(in vec2 uv) {
    uv *= 1.2;
    if (abs(uv.x) > 1.0) return 1;
    float t = asin(uv.x) / 2.0,
    a = iTime;
    return abs(uv.y - sin(3.0 * (t           ) + a)) < 0.1
        || abs(uv.y - sin(3.0 * (PI * 0.5 - t) + a)) < 0.1
        || abs(uv.y - sin(3.0 * (t + PI      ) + a)) < 0.1
        || abs(uv.y - sin(3.0 * (PI * 1.5 - t) + a)) < 0.1
        ? 0 : 1;
}

int sinWave(in vec2 uv) {
    return abs(uv.y) < sin((uv.x - iTime * 0.5) * 6.0) * 0.2 + 0.4 ? 0 : 1;
}

int message(in vec2 uv) {
    const int YOTABAITO[] = int[](89, 79, 84, 65, 66, 65, 73, 84, 79, 32);
    uv.y = clamp(uv.y * 0.5 + 0.5, 0.0, 1.0);
    uv.x = (uv.x + 10.0) + iTime * 0.5;
    int idx = int(floor(uv.x)) % YOTABAITO.length(),
    code = YOTABAITO[idx];
    vec2 base = vec2(code % 16, 15 - code / 16);
    uv.x = fract(uv.x) * 0.5 + 0.25;
    return textureLod(iChannel0, (base + uv) / 16.0, 0.0).x < 0.5 ? 1 : 0;
}

int picture(in ivec2 iuv) {
    vec2 uv = vec2(iuv) / DOT_RESOLUTION;
    float time = iTime / 4.0;
    vec2 rand = hash21(floor(time));
    int idx = int(rand.x * 5.0);
    int c = 0;
    switch (idx) {
        case 0: c = random(uv); break;
        case 1: c = circleWave(uv); break;
        case 2: c = lissajous(uv); break;
        case 3: c = sinWave(uv); break;
        default: c = message(uv); break;
    }
    return rand.y < 0.5 ? c : 1 - c;
}

float letter(in vec2 uv, in int code) {
    uv = ((uv - 0.5) * 0.99 + 0.5 + vec2(code, 12)) / 16.0;
    float dist = textureLod(iChannel0, uv, 0.0).w;
    return (clamp(0.1 / (dist * dist * dist), 0.2, 1.0) - 0.2) / 0.8;
}

void mainImage(out vec4 O, in vec2 U) {
    vec2 r = iResolution.xy;
    vec2 uv = DOT_RESOLUTION * (U + U - r) / r.y + vec2(.5, 0);
    int code = picture(ivec2(floor(uv)));
    float c = letter(fract(uv), code);
    O = vec4(c * c * c, c, c * c, 1);
}