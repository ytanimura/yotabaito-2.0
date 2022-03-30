#iChannel0 "../dist/codepage12.png"

void mainImage(out vec4 O, in vec2 U) {
	float dist = texture(iChannel0, U * 3.0 / iResolution.xy).w;
	float c = (clamp(0.1 / (dist * dist * dist), 0.01, 1.0) - 0.2) / 0.8;
	O = vec4(c * c * c, c, c * c, 1);
}