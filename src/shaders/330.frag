#version 330

uniform sampler2D tex;

in vec2 g_tex_coord;
in vec4 g_color;

out vec4 f_color;

void main() {
    f_color = texture(tex, g_tex_coord) * g_color;
}
