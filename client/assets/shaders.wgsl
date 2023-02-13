struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(in.pos, 1.0);
    out.vert_pos = in.pos;
    out.tex_coords = in.tex_coords;

    return out;
}

@group(0) @binding(0)
var diffuse: texture_2d<f32>;
@group(0) @binding(1)
var diffuse_sampler: sampler;
// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(diffuse, diffuse_sampler, in.tex_coords);
}
