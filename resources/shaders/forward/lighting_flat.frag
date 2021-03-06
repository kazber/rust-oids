#version 150 core

#define MAX_NUM_TOTAL_LIGHTS 16

const float PI = 3.1415926535897932384626433832795;
const float PI_2 = 1.57079632679489661923;

layout (std140) uniform cb_FragmentArgs {
	int u_LightCount;
};

struct Light {
	vec4 propagation;
	vec4 center;
	vec4 color;
};

layout (std140) uniform u_Lights {
	Light light[MAX_NUM_TOTAL_LIGHTS];
};

layout (std140) uniform cb_MaterialArgs {
	uniform vec4 u_Emissive;
	uniform vec4 u_Effect;
};

in VertexData {
	vec4 Position;
	vec3 Normal;
	mat3 TBN;
	vec2 TexCoord;
}v_In;

out vec4 o_Color;

void main() {
	vec4 kd = vec4(0.2, 0.2, 0.2, 1.0);
	vec4 ks = vec4(1.0, 1.0, 1.0, 1.0);
	vec4 kp = vec4(64.0, 32.0, 64.0, 1.0);

	float dx = 2 * clamp(v_In.TexCoord.x, 0, 1) - 1;
	float dy = 2 * clamp(v_In.TexCoord.y, 0, 1) - 1;
	float r = min(1, dx * dx + dy * dy);

	float f = clamp(u_Effect.x * 2, 0, 1);
	float e = clamp(abs(cos(r - u_Effect.y) + sin(dy - 2 * u_Effect.y)), 0, 1);

	o_Color = u_Emissive * e * f;
}
