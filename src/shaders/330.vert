#version 330

uniform mat4 matrix;

in vec3 position;
in vec4 color;
in vec2 tex_coord;

out vec2 v_tex_coord;
out vec4 v_color;

void main() {
    gl_Position = matrix * vec4(position.xy * 32, 0.0, 1.0);
    v_tex_coord = tex_coord;
    v_color = color;
}
