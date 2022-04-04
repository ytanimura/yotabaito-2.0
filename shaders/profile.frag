#iChannel0 "../resources/selfie.jpg"

// Post effect assortment
// Copyright © 2022 Dr. Yoshinori Tanimura
// MIT License (photograph texture is not included)

// Hash without Sine https://www.shadertoy.com/view/4djSRW
vec2 hash23(vec3 p3) {
    p3 = fract(p3 * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}

vec4 getColor(in vec2 U) {
    U.x = (U.x - 0.5 * iResolution.x / iResolution.y)
        / iChannelResolution[0].x
        * iChannelResolution[0].y
        + 0.5;
    return textureLod(iChannel0, fract(U), 0.0);
}

vec2 getNearestSample(in vec2 U) {
    const float TILE_RESOLUTION = 40.0;
    float time = iTime * 0.2;
    float it = floor(time), ft = fract(time);

    vec2 V = U * TILE_RESOLUTION,
    V0 = floor(V) + 0.5;
    float miniDist = 100.0;
    vec2 Vmini = vec2(0);
    for (int i = 0; i < 25; i++) {
        vec2 V0 = V0 + vec2(i % 5, i / 5) - 2.0;
        vec2 disp = mix(
            hash23(vec3(V0, it)),
            hash23(vec3(V0, it + 1.0)),
            ft
        );
        V0 += 4.0 * disp - 2.0;
        float dist = distance(V, V0);
        if (miniDist > dist) {
            miniDist = dist;
            Vmini = V0;
        }
    }
    return Vmini / TILE_RESOLUTION;
}

// voronoi mosaic
vec4 voronoi(in vec2 U) {
    U = getNearestSample(U);
    return getColor(U);
}

void mainImage(out vec4 O, in vec2 U) {
    U /= iResolution.y;
    O = voronoi(U);
}
