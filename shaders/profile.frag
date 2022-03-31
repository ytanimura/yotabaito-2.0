#iChannel0 "../dist/2f3c38f3d096272ba1629b9fd71b2dbeea8df59b516e3ec2385028dbf67e007c.jpg"

// Photo effects
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

vec4 voronoi(in vec2 U) {
    U = getNearestSample(U);
    return getColor(U);
}

float luminance(in vec3 c) {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

vec4 laplacian(in vec2 U) {
    vec2 disp = mix(
        hash23(vec3(U, floor(iTime))),
        hash23(vec3(U, floor(iTime) + 1.0)),
        fract(iTime)
    );
    U += 0.002 * disp - 0.001;
    vec3 e = vec3(1,1,0) / iResolution;
    vec4 col = -8.0 * getColor(U)
            + getColor(U - e.xz - e.zy)
            + getColor(U - e.xz)
            + getColor(U - e.xz + e.zy)
            + getColor(U - e.zy)
            + getColor(U + e.zy)
            + getColor(U + e.xz - e.zy)
            + getColor(U + e.xz)
            + getColor(U + e.xz + e.zy);
    float c = smoothstep(0.0, 1.0, luminance(col.xyz));
    return vec4(vec3(1.0 - c), 1);
}

vec4 points(in vec2 U) {
    float r = hash23(vec3(floor((U * iResolution.y + iTime * 5.0) / 2.0), iTime)).x;
    float c = luminance(getColor(U).xyz);
    return vec4(vec3(r > c ? 0.0 : 1.0), 1.0);
}

void mainImage(out vec4 O, in vec2 U) {
    U /= iResolution.y;
    int h = int(iTime / 4.0) % 3;
    switch (h) {
        case 0: O = voronoi(U); break;
        case 1: O = points(U); break;
        default: O = laplacian(U); break;
    }
}
