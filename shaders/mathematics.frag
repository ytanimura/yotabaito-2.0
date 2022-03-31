// Random Primitives
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

#define rotate3D(axis, angle) mat3(\
    axis[0] * axis[0] * (1.0 - cos(angle)) + cos(angle),\
    axis[0] * axis[1] * (1.0 - cos(angle)) + axis[2] * sin(angle),\
    axis[0] * axis[2] * (1.0 - cos(angle)) - axis[1] * sin(angle),\
    axis[0] * axis[1] * (1.0 - cos(angle)) - axis[2] * sin(angle),\
    axis[1] * axis[1] * (1.0 - cos(angle)) + cos(angle),\
    axis[1] * axis[2] * (1.0 - cos(angle)) + axis[0] * sin(angle),\
    axis[0] * axis[2] * (1.0 - cos(angle)) + axis[1] * sin(angle),\
    axis[1] * axis[2] * (1.0 - cos(angle)) - axis[0] * sin(angle),\
    axis[2] * axis[2] * (1.0 - cos(angle)) + cos(angle)\
)

float microfacet(vec3 normal, vec3 cameraDir, vec3 lightDir, float metal, float roughness) {
    vec3 middle = normalize(cameraDir + lightDir);
    float dotCN = clamp(-dot(cameraDir, normal), 0.0, 1.0);
    float dotLN = clamp(-dot(lightDir, normal), 0.0, 1.0);
    float dotNM = clamp(-dot(normal, middle), 0.0, 1.0);
    float dotCM = clamp(dot(cameraDir, middle), 0.0, 1.0);
    float alpha = roughness * roughness;
    
    // diffuse BRDF
    float diffuse = 1.0 - metal;
    
    // microfacet distribution
    float alpha2 = alpha * alpha;
    float tmp = 1.0 - dotNM * dotNM * (1.0 - alpha2);
    float distribution = alpha2 / (tmp * tmp);

    // schlick approxy & geometric decay
    float alp = alpha * 0.5;
    float sal = dotLN / (dotLN * (1.0 - alp) + alp);
    float sac = dotCN / (dotCN * (1.0 - alp) + alp);
    float decay = sal * sac;

    // fresnel
    float c = 1.0 - dotCM;
    c = c * c * c * c * c;
    float fresnel = metal + (1.0 - metal) * c;

    // specular BRDF
    tmp = 4.0 * dotCN * dotLN;
    float specular = distribution * decay / tmp * fresnel;
    if (tmp < 0.0001) specular = 0.0;
    
    return (diffuse + specular) * dotLN;
}

/******************** SDFs ********************/
// https://iquilezles.org/www/articles/distfunctions/distfunctions.htm

float sdSphere(in vec3 p, in float r) {
    return length(p) - 1.0;
}

float sdTorus(in vec3 p, in vec2 t) {
  vec2 q = vec2(length(p.xy) - t.x, p.z);
  return length(q) - t.y;
}

float sdBox(in vec3 p, in float a) {
  vec3 q = abs(p) - a;
  return length(max(q, 0.0)) + min(max(q.x,max(q.y,q.z)), 0.0);
}

float sdStar5(in vec2 p, in float r, in float rf) {
    const vec2 k1 = vec2(0.809016994375, -0.587785252292);
    const vec2 k2 = vec2(-k1.x,k1.y);
    p.x = abs(p.x);
    p -= 2.0*max(dot(k1,p),0.0)*k1;
    p -= 2.0*max(dot(k2,p),0.0)*k2;
    p.x = abs(p.x);
    p.y -= r;
    vec2 ba = rf*vec2(-k1.y,k1.x) - vec2(0,1);
    float h = clamp( dot(p,ba)/dot(ba,ba), 0.0, r );
    return length(p-ba*h) * sign(p.y*ba.x-p.x*ba.y);
}

float sdOctahedron( vec3 p, float s) {
  p = abs(p);
  float m = p.x+p.y+p.z-s;
  vec3 q;
       if( 3.0*p.x < m ) q = p.xyz;
  else if( 3.0*p.y < m ) q = p.yzx;
  else if( 3.0*p.z < m ) q = p.zxy;
  else return m*0.57735027;
    
  float k = clamp(0.5*(q.z-q.y+s),0.0,s); 
  return length(vec3(q.x,q.y-s+k,q.z-k)); 
}

/******************** Main ********************/
float preDist(in vec3 p, uint distId) {
    switch (distId) {
        case 0u: return sdSphere(p, 1.0);
        case 1u: return sdTorus(p, vec2(0.8, 0.2));
        case 2u: return sdBox(p - vec3(0,0.15,0), 1.0 / sqrt(2.0)) - 0.01;
        case 3u: return length(vec2(max(sdStar5(p.xy, 1.0, 0.4), 0.0), max(abs(p.z) - 0.1, 0.0))) - 0.01;
        case 4u: return sdOctahedron(p, 1.0);
        default: return 1.0;
    }
}

float sDist(in vec3 p) {
    float ft = fract(iTime / 5.0);
    float it = floor(iTime / 5.0);
    uint distId = uint(hash11(it + 0.13) * 5.0);
    float dist = 0.0;
    if (ft < 0.9) {
        dist = preDist(p, distId);
    } else {
        uint nextDistId = uint(hash11(it + 1.0 + 0.13) * 5.0);
        dist = mix(
            preDist(p, distId),
            preDist(p, nextDistId),
            smoothstep(0.9, 1.0, ft)
        );
    }
    return dist;
}

vec3 calcNormal(in vec3 p) {
    const vec2 h = vec2(1.0e-4, 0);
    return normalize(vec3(
        sDist(p + h.xyy) - sDist(p - h.xyy),
        sDist(p + h.yxy) - sDist(p - h.yxy),
        sDist(p + h.yyx) - sDist(p - h.yyx)
    ));
}

void mainImage0(out vec4 fragColor, vec2 fragCoord) {
    vec3 pos = vec3(3.0 * sin(iTime), 1.25, 3.0 * cos(iTime));
    Camera camera = Camera(
        pos,
        -normalize(pos),
        vec3(0,1,0),
        PI / 4.0,
        iResolution.x / iResolution.y
    );
    Ray ray = cameraRay(camera, fragCoord / iResolution.xy);

    float dist = 0.0, dist0 = 0.0;
    vec3 p;
    for (int i = 0; i < 128; i++) {
        p = ray.origin + dist * ray.direction;
        dist0 = sDist(p), dist += dist0;
        if (dist0 < 1.0e-4 || dist > 5.0) break;
    }

    vec3 col = pow(vec3(144, 215, 236) / 255.0, vec3(2.2));
    if (dist < 5.0) {
        vec3 normal = calcNormal(p);
        col = -dot(normal, ray.direction) * vec3(0.8, 0.9, 1.0);
    }

    fragColor = vec4(col,1);
}

// smart anti-aliasing
// reference: https://shadertoyunofficial.wordpress.com/2021/03/09/advanced-tricks/
void mainImage(out vec4 O, in vec2 U) {
    mainImage0(O, U);    
    if (fwidth(length(O.xyz)) > 0.01) {
        vec4 o;
        for (int k = 0; k < 4; k++) {
              mainImage0(o,U + (vec2(k % 2, k / 2) - 0.5) / 1.5);
              O += o;
        }
        O /= 5.0;
    }
    O.xyz = pow(O.xyz, vec3(.4545));
    O.w = 1.0;
}
