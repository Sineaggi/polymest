[[location(0)]]
var<in> in_position: vec4<f32>;
[[location(1)]]
var<in> in_tex_coord_vs: vec2<f32>;
[[location(0)]]
var<out> out_tex_coord: vec2<f32>;
[[builtin(position)]]
var<out> out_position: vec4<f32>;

[[stage(vertex)]]
fn vs_main() {
    out_tex_coord = in_tex_coord_vs;
    out_position = vec4<f32>(in_position * 2.0 - 1.0, 0.0, 1.0);
}

[[location(0)]]
var<in> f_texcoord: vec2<f32>;
[[location(0)]]
var<out> out_color: vec4<f32>;
[[group(0), binding(1)]]
var u_color: texture_2d<f32>;
[[group(0), binding(2)]]
var u_sampler: sampler;

[[stage(fragment)]]
fn fs_main() {
    out_color = textureSample(u_color, u_sampler, f_texcoord);
}
