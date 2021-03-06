#version 150 core

layout (std140) uniform cb_CameraArgs {
	uniform mat4 u_Proj;
	uniform mat4 u_View;
};

layout (std140) uniform cb_ModelArgs {
	uniform mat4 u_Model;
};

in vec3 a_Pos;
in vec3 a_Normal;
in vec3 a_Tangent;
in vec2 a_TexCoord;

out VertexData {
	vec4 Position;
	vec3 Normal;
	mat3 TBN;
	vec2 TexCoord;
}v_Out;

void main() {
	v_Out.Position = u_Model * vec4(a_Pos, 1.0);
	mat3 model = mat3(u_Model);
	vec3 normal = normalize(model * a_Normal);

	v_Out.Normal = normal;
	vec3 tangent = normalize(model * a_Tangent);
	vec3 bitangent = cross(normal, tangent);

	v_Out.TBN = mat3(tangent, bitangent, normal);

	v_Out.TexCoord = a_TexCoord;
	gl_Position = u_Proj * u_View * v_Out.Position;
}

