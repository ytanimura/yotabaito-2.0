// Main theme of yotabaito with TV filter
// Copyright © 2022 Dr. Yoshinori Tanimura
// Attribution 4.0 International (CC BY 4.0)

/******************** Hash ********************/
// Hash without Sine https://www.shadertoy.com/view/4djSRW
float hash11(float p) {
    p = fract(p * .1031);
    p *= p + 33.33;
    p *= p + p;
    return fract(p);
}
float hash12(vec2 p) {
    vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
float hash13(vec3 p3) {
    p3  = fract(p3 * .1031);
    p3 += dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}
vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}
vec2 hash23(vec3 p3) {
    p3 = fract(p3 * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}

/******************** 3D ToolKit ********************/
//https://www.shadertoy.com/view/ftGXzt
const float PI = 3.141592653;
struct Camera {
    vec3 position;
    vec3 direction;
    vec3 up_direction; // not require dot(direction, up_direction) == 0
    float fov;
    float aspect; // x / y
};

struct Ray {
    vec3 origin;
    vec3 direction;
};

// perspective camera ray, uv = fragCoord / iResolution.xy
// cf: https://qiita.com/aa_debdeb/items/301dfc54788f1219b554
Ray cameraRay(in Camera camera, in vec2 uv) {
    uv = uv * 2.0 - 1.0;
    float h = tan(camera.fov * 0.5);
    float w = h * camera.aspect;
    vec3 right = normalize(cross(camera.direction, camera.up_direction));
    vec3 up = normalize(cross(right, camera.direction));
    vec3 direction = normalize(right * w * uv.x + up * h * uv.y + camera.direction);
    Ray ray;
    ray.origin = camera.position;
    ray.direction = direction;
    return ray;
}

// Rodrigues' rotation formula
mat3 rotate3D(vec3 axis, float angle) {
    float c = cos(angle), s = sin(angle);
    return mat3(
        axis[0] * axis[0] * (1.0 - c) + c,
        axis[0] * axis[1] * (1.0 - c) + axis[2] * s,
        axis[0] * axis[2] * (1.0 - c) - axis[1] * s,
        axis[0] * axis[1] * (1.0 - c) - axis[2] * s,
        axis[1] * axis[1] * (1.0 - c) + c,
        axis[1] * axis[2] * (1.0 - c) + axis[0] * s,
        axis[0] * axis[2] * (1.0 - c) + axis[1] * s,
        axis[1] * axis[2] * (1.0 - c) - axis[0] * s,
        axis[2] * axis[2] * (1.0 - c) + c
    );
}

/******************** Main ********************/
const float FAR = 12.0;

// https://www.iquilezles.org/www/articles/intersectors/intersectors.htm
float roundedboxIntersect(in Ray ray, in vec3 size, in float rad ) {
    vec3 rd = ray.direction, ro = ray.origin;
    // bounding box
    vec3 m = 1.0/rd;
    vec3 n = m*ro;
    vec3 k = abs(m)*(size+rad);
    vec3 t1 = -n - k;
    vec3 t2 = -n + k;
    float tN = max( max( t1.x, t1.y ), t1.z );
    float tF = min( min( t2.x, t2.y ), t2.z );
    if( tN>tF || tF<0.0) return -1.0;
    float t = tN;

    // convert to first octant
    vec3 pos = ro+t*rd;
    vec3 s = sign(pos);
    ro  *= s;
    rd  *= s;
    pos *= s;
        
    // faces
    pos -= size;
    pos = max( pos.xyz, pos.yzx );
    if( min(min(pos.x,pos.y),pos.z) < 0.0 ) return t;

    // some precomputation
    vec3 oc = ro - size;
    vec3 dd = rd*rd;
    vec3 oo = oc*oc;
    vec3 od = oc*rd;
    float ra2 = rad*rad;

    t = 1e20;        

    // corner
    {
    float b = od.x + od.y + od.z;
    float c = oo.x + oo.y + oo.z - ra2;
    float h = b*b - c;
    if( h>0.0 ) t = -b-sqrt(h);
    }
    // edge X
    {
    float a = dd.y + dd.z;
    float b = od.y + od.z;
    float c = oo.y + oo.z - ra2;
    float h = b*b - a*c;
    if( h>0.0 )
    {
        h = (-b-sqrt(h))/a;
        if( h>0.0 && h<t && abs(ro.x+rd.x*h)<size.x ) t = h;
    }
    }
    // edge Y
    {
    float a = dd.z + dd.x;
    float b = od.z + od.x;
    float c = oo.z + oo.x - ra2;
    float h = b*b - a*c;
    if( h>0.0 )
    {
        h = (-b-sqrt(h))/a;
        if( h>0.0 && h<t && abs(ro.y+rd.y*h)<size.y ) t = h;
    }
    }
    // edge Z
    {
    float a = dd.x + dd.y;
    float b = od.x + od.y;
    float c = oo.x + oo.y - ra2;
    float h = b*b - a*c;
    if( h>0.0 )
    {
        h = (-b-sqrt(h))/a;
        if( h>0.0 && h<t && abs(ro.z+rd.z*h)<size.z ) t = h;
    }
    }

    if( t>1e19 ) t=-1.0;
    
    return t;
}

// https://www.shadertoy.com/view/WlSXRW
vec3 roundedboxNormal( in vec3 pos, in vec3 siz, in float rad ) {
    return sign(pos)*normalize(max(abs(pos)-siz,0.0));
}

// Random unit vector
// https://qiita.com/aa_debdeb/items/e416ae8a018692fc07eb
vec3 randomAxis(vec2 gen) {
    vec2 uv = hash22(gen);
    float z = 2.0 * uv.x - 1.0;
    float t = 2.0 * PI * uv.y;
    return vec3(
        sqrt(1.0 - z * z) * cos(t),
        sqrt(1.0 - z * z) * sin(t),
        z
    );
}

float rayFoward(in Ray ray, in vec3 p) {
    vec3 m = 1.0 / ray.direction,
    n = m * (ray.origin - floor(p) - 0.5),
    k = abs(m) * 0.5,
    t2 = -n + k;
    return min(min(t2.x, t2.y), t2.z) + 1.0e-2;
}

void mainImage0(out vec4 O, in vec2 U) {
    float ft = fract(iTime * 0.1), it = floor(iTime * 0.1);
    Camera camera = Camera(
        vec3(0, 0, ft),
        vec3(0, 0, 1),
        vec3(0, 1, 0),
        PI / 4.0,
        iResolution.x / iResolution.y
    );
    Ray ray = cameraRay(camera, U);
    vec3 base = vec3(0, 0, it);

    float dist = -1.0, t = 0.0; vec3 normal;
    for (int i = 0; i < 32 && t < FAR; i++) {
        vec3 p = ray.origin + t * ray.direction;
        vec2 rand = hash23(floor(p + base));
        if (rand.x > 0.9) {
            vec3 axis = randomAxis(hash23(floor(p + base)));
            mat3 mat = rotate3D(axis, iTime + rand.y);
            Ray ray0 = Ray(
                mat * (ray.origin - floor(p) - 0.5),
                normalize(mat * ray.direction)
            );
            dist = roundedboxIntersect(ray0, vec3(0.15), 0.02);
            if (dist != -1.0) {
                vec3 p = ray0.origin + dist * ray0.direction;
                normal = transpose(mat)
                    * roundedboxNormal(p, vec3(0.15), 0.02);
                break;
            }
        }
        t = rayFoward(ray, p);
    }

    vec3 col = vec3(0.8, 0.9, 1.0);
    if (dist != -1.0 && t < FAR) {
        float c = -dot(ray.direction, normal);
        c = clamp(c, 0.0, 1.0);
        float k = dist / FAR;
        if (k < 1.0) {
            k *= k;
            k = smoothstep(0.7, 1.0, k);
            vec3 material = pow(vec3(144, 215, 236) / 255.0, vec3(2.2));
            col = (1.0 - k) * c * material + k * col;
        }
    }

    O = vec4(col, 1);
}

float fbm(vec3 x) {
    float t = 0.0;
    for( int i=0; i < 5; i++ ) {
        t += hash13(x);
        x *= 2.0;
    }
    return t;
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 r = floor(iResolution.xy) / 3.0;
    int idx = int(fragCoord.x) % 3;
    vec2 uv = floor(fragCoord / 3.0) / r;
    float disp = clamp(fbm(vec3(uv.y * 10.0, iTime * 100.0, idx)) / 5.0 - 0.75, 0.0, 1.0);
    
    float c = hash12(floor(fragCoord / 3.0) + vec2(114, 514) * iTime) * 0.1;
    
    vec4 O;
    mainImage0(O, uv - vec2(disp, 0.0));
    fragColor[idx] = pow(O[idx] + c, .4545);
    fragColor.w = 1.0;
}
