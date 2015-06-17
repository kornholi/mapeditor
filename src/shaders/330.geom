#version 330

uniform mat4 matrix;
layout(points) in;
layout(triangle_strip, max_vertices = 4) out;

uniform float texture_size = 1.0 / (2048./32.);

in vec2 v_tex_coord[];
in vec4 v_color[];

out vec2 g_tex_coord;
out vec4 g_color;

void main(void)
{
    int i;
    for(i=0; i < gl_in.length(); i++)
	{
        vec4 in_pos = gl_in[i].gl_Position;
		vec2 in_texture = v_tex_coord[i];
		g_color = v_color[i];

        gl_Position = in_pos;
        g_tex_coord = in_texture + vec2(0, texture_size);
	    EmitVertex();

        gl_Position = in_pos + matrix * vec4(0, 32, 0, 0);
        g_tex_coord = in_texture + vec2(0, 0); 
	    EmitVertex();

        gl_Position = in_pos + matrix * vec4(32, 0, 0, 0);
        g_tex_coord = in_texture + vec2(texture_size, texture_size);
	    EmitVertex();

        gl_Position = in_pos + matrix * vec4(32, 32, 0, 0);
        g_tex_coord = in_texture + vec2(texture_size, 0);
	    EmitVertex();

        EndPrimitive();
	}
}